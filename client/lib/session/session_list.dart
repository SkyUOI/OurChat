import 'dart:async';
import 'dart:math';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/notification_service.dart';
import 'package:ourchat/core/session.dart' as core_session;
import 'package:ourchat/main.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'state.dart';
import 'session_list_item.dart';
import 'new_session_dialog.dart';
import 'join_session_dialog.dart';
import 'session_tab.dart';

class SessionList extends ConsumerStatefulWidget {
  const SessionList({super.key});

  @override
  ConsumerState<SessionList> createState() => _SessionListState();
}

class _SessionListState extends ConsumerState<SessionList> {
  Timer? _debounceTimer = Timer(Duration.zero, () {}); // Search debounce timer
  bool search = false; // Searching
  String searchKeyword = "";
  // Owning controller for the search box: on web the semantics-layer input
  // element is recreated when the results section appears below this field,
  // and without an authoritative controller the framework re-wrote an empty
  // value into it, wiping the user's input mid-typing.
  final TextEditingController _searchController = TextEditingController();

  late final SessionNotifier _sessionNotifier;
  OurChatEventSystem? _eventSystem;
  AccountKey? _boundKey;

  void _onMsgReceived(UserMsg eventObj) {
    _sessionNotifier.receiveMsg(eventObj);
    unawaited(_notifyNewMessage(eventObj));
    // Refresh sender account info asynchronously
    final sid = _boundKey?.serverId;
    if (sid != null) {
      ref
          .read(ourChatAccountProvider(sid, eventObj.senderId!).notifier)
          .getAccountInfo();
    }
  }

  /// Fire the system notification + tray flash for an incoming message when
  /// the user is not already looking at that conversation (issue #199).
  /// Best effort: never throws into the event-stream handler.
  Future<void> _notifyNewMessage(UserMsg msg) async {
    final key = _boundKey;
    final sessionId = msg.sessionId;
    if (key == null || sessionId == null) return;
    try {
      final config = ref.read(configProvider);
      final sessionState = ref.read(sessionProvider);

      // Conversation title: session name, else the 2-person display name,
      // else the sender's username, else a generic label.
      final sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(key.serverId, sessionId).notifier,
      );
      // Warm the session cache in the background so later notifications can
      // use the real name; never block the notification on the network.
      unawaited(sessionNotifier.getSessionInfo().catchError((_) => false));
      final sessionData = ref.read(
        core_session.ourChatSessionProvider(key.serverId, sessionId),
      );
      String title = sessionData.name;
      if (title.isEmpty) title = sessionData.displayName ?? '';
      if (title.isEmpty && msg.senderId != null) {
        title = ref
            .read(ourChatAccountProvider(key.serverId, msg.senderId!))
            .username;
      }
      if (title.isEmpty) title = l10n.newMessage;

      // Plain-text preview of the message body (replaced by a generic string
      // by the privacy switch when it is off).
      var preview = MarkdownToText.convert(msg.markdownText, l10n);
      if (preview.length > 50) {
        preview = "${preview.substring(0, 50)}...";
      }

      await notifyNewMessage(
        msg,
        notificationService: ref.read(ourChatNotificationServiceProvider),
        currentSessionId: sessionState.currentSessionId,
        appInForeground: appInForeground,
        thisAccountId: key.accountId,
        showContent: config.notificationShowMessageContent,
        title: title,
        preview: preview,
        genericBody: l10n.newMessage,
        flashTray: isDesktopPlatform ? startFlashTray : null,
      );
    } catch (e) {
      logger.w("new message notification failed: $e");
    }
  }

  void _onJoinApproved(OurChatEvent eventObj) {
    // A join request of ours was accepted — reload the session list so the
    // new conversation appears without a manual refresh (issue #289).
    _loadSessions();
  }

  void _onVoteNotification(OurChatEvent eventObj) {
    // Recall-vote lifecycle update (issue #33): mirror it into the session
    // state so the vote banner in the session tab stays current.
    if (eventObj is! RecallVoteNotificationEvent) return;
    _sessionNotifier.updateSessionVote(
      RecallVoteData(
        voteId: eventObj.voteId,
        sessionId: eventObj.sessionId!,
        targetMsgId: eventObj.targetMsgId,
        initiatorId: eventObj.initiatorId,
        yesCount: eventObj.yesCount,
        noCount: eventObj.noCount,
        eligibleCount: eventObj.eligibleCount,
        deadline: eventObj.deadline,
        settled: eventObj.settled,
        passed: eventObj.passed,
      ),
    );
  }

  Future<void> _loadSessions() async {
    await ref.read(sessionProvider.notifier).loadSessions();
  }

  @override
  void initState() {
    super.initState();
    _sessionNotifier = ref.read(sessionProvider.notifier);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadSessions();
    });
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    _searchController.dispose();
    _eventSystem?.removeListener(
      FetchMsgsResponse_RespondEventType.msg,
      _onMsgReceived,
    );
    _eventSystem?.removeListener(
      FetchMsgsResponse_RespondEventType.allowUserJoinSessionNotification,
      _onJoinApproved,
    );
    _eventSystem?.removeListener(
      FetchMsgsResponse_RespondEventType.recallVoteNotification,
      _onVoteNotification,
    );
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final activeKey = ref.watch(activeAccountProvider);
    var sessionState = ref.watch(sessionProvider);

    // (Re)bind the msg listener to the active account's event system.
    if (activeKey != null && activeKey != _boundKey) {
      _eventSystem?.removeListener(
        FetchMsgsResponse_RespondEventType.msg,
        _onMsgReceived,
      );
      _eventSystem?.removeListener(
        FetchMsgsResponse_RespondEventType.allowUserJoinSessionNotification,
        _onJoinApproved,
      );
      _eventSystem?.removeListener(
        FetchMsgsResponse_RespondEventType.recallVoteNotification,
        _onVoteNotification,
      );
      final newEventSystem = ref.read(
        ourChatEventSystemProvider(
          activeKey.serverId,
          activeKey.accountId,
        ).notifier,
      );
      newEventSystem.addListener(
        FetchMsgsResponse_RespondEventType.msg,
        _onMsgReceived,
      );
      newEventSystem.addListener(
        FetchMsgsResponse_RespondEventType.allowUserJoinSessionNotification,
        _onJoinApproved,
      );
      newEventSystem.addListener(
        FetchMsgsResponse_RespondEventType.recallVoteNotification,
        _onVoteNotification,
      );
      _eventSystem = newEventSystem;
      _boundKey = activeKey;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _loadSessions();
      });
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        return Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    // Search box
                    controller: _searchController,
                    decoration: InputDecoration(hintText: l10n.search),
                    onChanged: (value) {
                      setState(() {
                        searchKeyword = value;
                        search = false;
                      });
                      _debounceTimer!.cancel();
                      _debounceTimer = Timer(
                        const Duration(seconds: 1),
                        () => setState(() {
                          search =
                              true &&
                              value
                                  .isNotEmpty; // No input for one second -> search
                        }),
                      );
                    },
                  ),
                ),
                IconButton(
                  onPressed: () {
                    showDialog(
                      context: context,
                      builder: (context) {
                        return NewSessionDialog(sessionState: sessionState);
                      },
                    );
                  },
                  icon: const Icon(Icons.add),
                ), // Create session
              ],
            ),
            // The session list (normal view) and the search results are both
            // kept mounted in an IndexedStack: swapping this Column's children
            // with `if (search)` conditionals while the search field is
            // focused restructures Flutter web's semantics tree mid-edit, and
            // the engine then wipes the input (observed as onChanged('')).
            Expanded(
              child: IndexedStack(
                index: search ? 1 : 0,
                children: [
                  sessionState.sessionsLoading
                      ? Center(
                          child: CircularProgressIndicator(
                            color: Theme.of(context).primaryColor,
                          ),
                        )
                      : ListView.builder(
                          itemBuilder: (context, index) {
                            Int64 currentSessionId =
                                sessionState.sessionsList[index];
                            final sessionServerId =
                                sessionState
                                    .sessionServerIds[currentSessionId] ??
                                activeKey!.serverId;
                            final isForeign =
                                sessionServerId != activeKey!.serverId;
                            final currentSessionNotifier = ref.read(
                              core_session
                                  .ourChatSessionProvider(
                                    sessionServerId,
                                    currentSessionId,
                                  )
                                  .notifier,
                            );
                            String recentMsgText = "";
                            if (sessionState.sessionLatestMsg.containsKey(
                              currentSessionId,
                            )) {
                              final latestMsg = sessionState
                                  .sessionLatestMsg[currentSessionId]!;
                              final senderData = ref.read(
                                ourChatAccountProvider(
                                  sessionServerId,
                                  latestMsg.senderId!,
                                ),
                              );
                              recentMsgText =
                                  "${senderData.username}: ${MarkdownToText.convert(latestMsg.markdownText, l10n)}";
                              if (recentMsgText.length > 25) {
                                recentMsgText = recentMsgText.substring(
                                  0,
                                  min(25, recentMsgText.length),
                                );
                                recentMsgText += "...";
                              }
                            }
                            return SizedBox(
                              height: 80.0,
                              child: Padding(
                                padding: const EdgeInsets.only(top: 10.0),
                                child: ElevatedButton(
                                  style: ButtonStyle(
                                    shape: WidgetStateProperty.all(
                                      RoundedRectangleBorder(
                                        borderRadius: BorderRadius.circular(
                                          10.0,
                                        ),
                                      ),
                                    ),
                                  ),
                                  onPressed: () async {
                                    if (isForeign) {
                                      // Unified inbox: switch to the server that
                                      // owns this conversation before opening it.
                                      final inst = instanceForServer(
                                        ref.container,
                                        sessionServerId,
                                      );
                                      if (inst != null) {
                                        switchActive(ref, inst.key);
                                      }
                                    }
                                    final sid = ref.read(
                                      activeServerIdProvider,
                                    )!;
                                    final aid = ref.read(
                                      activeAccountIdProvider,
                                    )!;
                                    if (ref.read(screenModeProvider) ==
                                        ScreenMode.mobile) {
                                      Navigator.push(
                                        context,
                                        MaterialPageRoute(
                                          builder: (_) => TabWidget(),
                                        ),
                                      );
                                    }
                                    var records = await ref
                                        .read(
                                          ourChatEventSystemProvider(
                                            sid,
                                            aid,
                                          ).notifier,
                                        )
                                        .getSessionEvent(
                                          currentSessionId,
                                          fetchFromServer: true,
                                        );
                                    ref
                                        .read(sessionProvider.notifier)
                                        .openSessionTab(
                                          currentSessionId,
                                          currentSessionNotifier
                                              .getDisplayName(),
                                          records: records,
                                        );
                                  },
                                  child: Row(
                                    mainAxisAlignment: MainAxisAlignment.start,
                                    children: [
                                      SizedBox(
                                        height: 40,
                                        width: 40,
                                        child: Image(
                                          image: AssetImage(
                                            "assets/images/logo.png",
                                          ),
                                        ),
                                      ),
                                      Expanded(
                                        child: Padding(
                                          padding: EdgeInsets.only(left: 8.0),
                                          child: Column(
                                            mainAxisAlignment:
                                                MainAxisAlignment.center,
                                            children: [
                                              Align(
                                                alignment: Alignment.centerLeft,
                                                widthFactor: 1.0,
                                                child: Row(
                                                  mainAxisSize:
                                                      MainAxisSize.min,
                                                  children: [
                                                    Flexible(
                                                      child: Text(
                                                        currentSessionNotifier
                                                            .getDisplayName(),
                                                        style: TextStyle(
                                                          fontSize: 20,
                                                          color:
                                                              Theme.of(context)
                                                                  .textTheme
                                                                  .labelMedium!
                                                                  .color,
                                                        ),
                                                        overflow: TextOverflow
                                                            .ellipsis,
                                                      ),
                                                    ),
                                                    if (isForeign)
                                                      Padding(
                                                        padding:
                                                            const EdgeInsets.only(
                                                              left: 6,
                                                            ),
                                                        child: _serverLabelChip(
                                                          sessionServerId,
                                                        ),
                                                      ),
                                                  ],
                                                ),
                                              ),
                                              if (sessionState.sessionLatestMsg
                                                  .containsKey(
                                                    currentSessionId,
                                                  ))
                                                Align(
                                                  alignment:
                                                      Alignment.centerLeft,
                                                  widthFactor: 1.0,
                                                  child: Text(
                                                    recentMsgText,
                                                    style: TextStyle(
                                                      color: Colors.grey,
                                                    ),
                                                    overflow:
                                                        TextOverflow.ellipsis,
                                                  ),
                                                ),
                                            ],
                                          ),
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            );
                          },
                          itemCount: sessionState.sessionsList.length,
                        ),
                  // Search results panel (hidden while the index points at the
                  // session list).
                  SingleChildScrollView(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Align(
                          alignment: Alignment.centerLeft,
                          child: Text(l10n.user),
                        ),
                        FutureBuilder(
                          future: search
                              ? searchAccount(
                                  thisAccountId,
                                  searchKeyword,
                                  context,
                                )
                              : null,
                          builder:
                              (BuildContext context, AsyncSnapshot snapshot) {
                                if (snapshot.connectionState !=
                                    ConnectionState.done) {
                                  return const SizedBox.shrink();
                                }
                                List<Int64> accountIds = snapshot.data ?? [];
                                if (accountIds.isEmpty) {
                                  return Padding(
                                    padding: const EdgeInsets.only(top: 5.0),
                                    child: Text(l10n.notFound(l10n.user)),
                                  );
                                }
                                return SizedBox(
                                  height: accountIds.length * 50,
                                  child: ListView.builder(
                                    itemBuilder: (context, index) {
                                      Int64 accountId = accountIds[index];
                                      final accountNotifier = ref.read(
                                        ourChatAccountProvider(
                                          activeKey!.serverId,
                                          accountId,
                                        ).notifier,
                                      );
                                      return SessionListItem(
                                        avatar: UserAvatar(
                                          imageUrl: accountNotifier.avatarUrl(),
                                        ),
                                        name: accountNotifier
                                            .getNameWithDisplayName(),
                                        onPressed: () {
                                          ref
                                              .read(sessionProvider.notifier)
                                              .openUserTab(
                                                accountId,
                                                l10n.userInfo,
                                              );
                                          if (ref.read(screenModeProvider) ==
                                              ScreenMode.mobile) {
                                            Navigator.push(
                                              context,
                                              MaterialPageRoute(
                                                builder: (_) => TabWidget(),
                                              ),
                                            );
                                          }
                                        },
                                      );
                                    },
                                    itemCount: accountIds.length,
                                  ),
                                );
                              },
                        ),
                        const Divider(),
                        Align(
                          alignment: Alignment.centerLeft,
                          child: Text(l10n.session),
                        ),
                        FutureBuilder(
                          future: search
                              ? searchSession(
                                  thisAccountId,
                                  searchKeyword,
                                  context,
                                )
                              : null,
                          builder: (context, snapshot) {
                            if (snapshot.connectionState !=
                                ConnectionState.done) {
                              return const SizedBox.shrink();
                            }
                            List<Int64> sessionIds = snapshot.data ?? [];
                            if (sessionIds.isEmpty) {
                              // Give a dedicated hint when the keyword looks
                              // like a session id but no session matches it
                              // (issue #289).
                              final isIdQuery =
                                  Int64.tryParseInt(searchKeyword) != null;
                              return Padding(
                                padding: const EdgeInsets.only(top: 5.0),
                                child: Text(
                                  isIdQuery
                                      ? l10n.sessionIdSearchNoResult
                                      : l10n.notFound(l10n.session),
                                ),
                              );
                            }
                            return SizedBox(
                              height: sessionIds.length * 50,
                              child: ListView.builder(
                                itemBuilder: (context, index) {
                                  Int64 sessionId = sessionIds[index];
                                  final sessionNotifier = ref.read(
                                    core_session
                                        .ourChatSessionProvider(
                                          activeKey!.serverId,
                                          sessionId,
                                        )
                                        .notifier,
                                  );
                                  return SessionListItem(
                                    avatar: Placeholder(),
                                    name: sessionNotifier.getDisplayName(),
                                    onPressed: () {
                                      final accountData = ref.read(
                                        ourChatAccountProvider(
                                          activeKey.serverId,
                                          thisAccountId!,
                                        ),
                                      );
                                      if (!accountData.sessions.contains(
                                        sessionId,
                                      )) {
                                        // Not a member yet: ask to join
                                        // instead of opening the conversation
                                        // (issue #289).
                                        showDialog(
                                          context: context,
                                          builder: (context) =>
                                              JoinSessionDialog(
                                                sessionId: sessionId,
                                              ),
                                        );
                                        return;
                                      }
                                      ref
                                          .read(sessionProvider.notifier)
                                          .openSessionTab(
                                            sessionId,
                                            sessionNotifier.getDisplayName(),
                                          );
                                      if (ref.read(screenModeProvider) ==
                                          ScreenMode.mobile) {
                                        Navigator.push(
                                          context,
                                          MaterialPageRoute(
                                            builder: (_) => TabWidget(),
                                          ),
                                        );
                                      }
                                    },
                                  );
                                },
                                itemCount: sessionIds.length,
                              ),
                            );
                          },
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }

  Future<List<Int64>> searchAccount(
    Int64? thisAccountId,
    String ocid,
    BuildContext context,
  ) async {
    final serverId = ref.read(activeServerIdProvider)!;
    List<Int64> matchAccountIds = [];
    BasicServiceClient stub = BasicServiceClient(
      ref.read(ourChatServerProvider).channel,
      interceptors: [],
    );

    // By OCID
    try {
      var res = await safeRequest(stub.getId, GetIdRequest(ocid: ocid), (
        grpc.GrpcError e,
      ) {
        showResultMessage(
          e.code,
          e.message,
          // getAccountInfo
          permissionDeniedStatus: l10n.permissionDenied("Get Account Info"),
          invalidArgumentStatus: l10n.internalError,
          notFoundStatus: "",
        );
      }, rethrowError: true);
      final notifier = ref.read(
        ourChatAccountProvider(serverId, res.id).notifier,
      );
      notifier.recreateStub();
      if (await notifier.getAccountInfo()) {
        matchAccountIds.add(res.id);
      }
    } catch (e) {
      // not found
    }

    // By username/display_name

    for (Int64 friendsId
        in ref.read(ourChatAccountProvider(serverId, thisAccountId!)).friends) {
      final notifier = ref.read(
        ourChatAccountProvider(serverId, friendsId).notifier,
      );
      notifier.recreateStub();
      if (await notifier.getAccountInfo() &&
          !matchAccountIds.contains(friendsId) &&
          notifier.getNameWithDisplayName().toLowerCase().contains(
            searchKeyword,
          )) {
        matchAccountIds.add(friendsId);
      }
    }

    return matchAccountIds;
  }

  Future searchSession(
    Int64? thisAccountId,
    String searchKeyword,
    BuildContext context,
  ) async {
    final serverId = ref.read(activeServerIdProvider)!;
    Int64? sessionId = Int64.tryParseInt(searchKeyword);
    List<Int64> matchSessions = [];

    if (sessionId != null) {
      // By sessionId

      core_session.OurChatSession sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(serverId, sessionId).notifier,
      );
      try {
        if (await sessionNotifier.getSessionInfo()) {
          matchSessions.add(sessionId);
        }
      } catch (e) {
        // do nothing
      }
    }

    // by name/description
    final accountData = ref.read(
      ourChatAccountProvider(serverId, thisAccountId!),
    );
    for (Int64 sid in accountData.sessions) {
      core_session.OurChatSession sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(serverId, sid).notifier,
      );
      await sessionNotifier.getSessionInfo();
      final sessionData = ref.read(
        core_session.ourChatSessionProvider(serverId, sid),
      );
      if ((sessionData.description.toLowerCase().contains(searchKeyword) ||
              sessionData.name.toLowerCase().contains(searchKeyword) ||
              sessionNotifier.getDisplayName().toLowerCase().contains(
                searchKeyword,
              )) &&
          !matchSessions.contains(sid)) {
        matchSessions.add(sid);
      }
    }
    return matchSessions;
  }

  /// A small badge showing which server a conversation belongs to (used in
  /// unified-inbox mode for conversations from other servers).
  Widget _serverLabelChip(String serverId) {
    final cfg = ref.read(configProvider);
    String label = serverId;
    for (final s in cfg.servers) {
      if (s.uniqueIdentifier == serverId) {
        if (s.label != null && s.label!.isNotEmpty) {
          label = s.label!;
        } else {
          label = s.host;
        }
        break;
      }
    }
    if (label.length > 8) label = label.substring(0, 8);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
      decoration: BoxDecoration(
        color: Colors.grey.shade300,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(label, style: const TextStyle(fontSize: 10)),
    );
  }
}
