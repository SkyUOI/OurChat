import 'dart:async';
import 'dart:math';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/session.dart' as core_session;
import 'package:ourchat/main.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'state.dart';
import 'session_list_item.dart';
import 'new_session_dialog.dart';
import 'session_tab.dart';

class SessionList extends ConsumerStatefulWidget {
  const SessionList({super.key});

  @override
  ConsumerState<SessionList> createState() => _SessionListState();
}

class _SessionListState extends ConsumerState<SessionList> {
  Timer? _debounceTimer = Timer(Duration.zero, () {}); // 搜索timer
  bool search = false; // 搜索中
  String searchKeyword = "";

  late final OurChatEventSystem _eventSystem;
  late final SessionNotifier _sessionNotifier;

  void _onMsgReceived(UserMsg eventObj) {
    _sessionNotifier.receiveMsg(eventObj);
    // Refresh sender account info asynchronously
    ref
        .read(ourChatAccountProvider(eventObj.senderId!).notifier)
        .getAccountInfo();
  }

  Future<void> _loadSessions() async {
    await ref.read(sessionProvider.notifier).loadSessions();
  }

  @override
  void initState() {
    super.initState();
    _eventSystem = ref.read(ourChatEventSystemProvider.notifier);
    _sessionNotifier = ref.read(sessionProvider.notifier);
    _eventSystem.addListener(
      FetchMsgsResponse_RespondEventType.msg,
      _onMsgReceived,
    );
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadSessions();
    });
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    _eventSystem.removeListener(
      FetchMsgsResponse_RespondEventType.msg,
      _onMsgReceived,
    );
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var sessionState = ref.watch(sessionProvider);
    return LayoutBuilder(
      builder: (context, constraints) {
        return Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    // 搜索框
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
                          search = true && value.isNotEmpty; // 一秒内没输入，搜索
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
                ), // 创建会话
              ],
            ),
            if (search)
              Align(alignment: Alignment.centerLeft, child: Text(l10n.user)),
            if (search)
              FutureBuilder(
                future: searchAccount(thisAccountId, searchKeyword, context),
                builder: (BuildContext context, AsyncSnapshot snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    return CircularProgressIndicator(
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<Int64> accountIds = snapshot.data;
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
                          ourChatAccountProvider(accountId).notifier,
                        );
                        return SessionListItem(
                          avatar: UserAvatar(
                            imageUrl: accountNotifier.avatarUrl(),
                          ),
                          name: accountNotifier.getNameWithDisplayName(),
                          onPressed: () {
                            ref
                                .read(sessionProvider.notifier)
                                .openUserTab(accountId, l10n.userInfo);
                            if (ref.read(screenModeProvider) ==
                                ScreenMode.mobile) {
                              Navigator.push(
                                context,
                                MaterialPageRoute(builder: (_) => TabWidget()),
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
            if (search) const Divider(),
            if (search)
              Align(alignment: Alignment.centerLeft, child: Text(l10n.session)),
            if (search)
              FutureBuilder(
                future: searchSession(thisAccountId, searchKeyword, context),
                builder: (context, snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    return CircularProgressIndicator(
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<Int64> sessionIds = snapshot.data;
                  if (sessionIds.isEmpty) {
                    return Padding(
                      padding: const EdgeInsets.only(top: 5.0),
                      child: Text(l10n.notFound(l10n.session)),
                    );
                  }
                  return SizedBox(
                    height: sessionIds.length * 50,
                    child: ListView.builder(
                      itemBuilder: (context, index) {
                        Int64 sessionId = sessionIds[index];
                        final sessionNotifier = ref.read(
                          core_session
                              .ourChatSessionProvider(sessionId)
                              .notifier,
                        );
                        return SessionListItem(
                          avatar: Placeholder(),
                          name: sessionNotifier.getDisplayName(),
                          onPressed: () {
                            ref
                                .read(sessionProvider.notifier)
                                .openSessionTab(
                                  sessionId,
                                  sessionNotifier.getDisplayName(),
                                );
                          },
                        );
                      },
                      itemCount: sessionIds.length,
                    ),
                  );
                },
              ),
            if (!search)
              Expanded(
                child: sessionState.sessionsLoading
                    ? Center(
                        child: CircularProgressIndicator(
                          color: Theme.of(context).primaryColor,
                        ),
                      )
                    : ListView.builder(
                        itemBuilder: (context, index) {
                          Int64 currentSessionId =
                              sessionState.sessionsList[index];
                          final currentSessionNotifier = ref.read(
                            core_session
                                .ourChatSessionProvider(currentSessionId)
                                .notifier,
                          );
                          String recentMsgText = "";
                          if (sessionState.sessionLatestMsg.containsKey(
                            currentSessionId,
                          )) {
                            final latestMsg = sessionState
                                .sessionLatestMsg[currentSessionId]!;
                            final senderData = ref.read(
                              ourChatAccountProvider(latestMsg.senderId!),
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
                                      borderRadius: BorderRadius.circular(10.0),
                                    ),
                                  ),
                                ),
                                onPressed: () async {
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
                                      .read(ourChatEventSystemProvider.notifier)
                                      .getSessionEvent(currentSessionId);
                                  ref
                                      .read(sessionProvider.notifier)
                                      .openSessionTab(
                                        currentSessionId,
                                        currentSessionNotifier.getDisplayName(),
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
                                              child: Text(
                                                currentSessionNotifier
                                                    .getDisplayName(),
                                                style: TextStyle(
                                                  fontSize: 20,
                                                  color: Theme.of(context)
                                                      .textTheme
                                                      .labelMedium!
                                                      .color,
                                                ),
                                                overflow: TextOverflow.ellipsis,
                                              ),
                                            ),
                                            if (sessionState.sessionLatestMsg
                                                .containsKey(currentSessionId))
                                              Align(
                                                alignment: Alignment.centerLeft,
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
      final notifier = ref.read(ourChatAccountProvider(res.id).notifier);
      notifier.recreateStub();
      if (await notifier.getAccountInfo()) {
        matchAccountIds.add(res.id);
      }
    } catch (e) {
      // not found
    }

    // By username/display_name

    for (Int64 friendsId
        in ref.read(ourChatAccountProvider(thisAccountId!)).friends) {
      final notifier = ref.read(ourChatAccountProvider(friendsId).notifier);
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
    Int64? sessionId = Int64.tryParseInt(searchKeyword);
    List<Int64> matchSessions = [];

    if (sessionId != null) {
      // By sessionId

      core_session.OurChatSession sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(sessionId).notifier,
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
    final accountData = ref.read(ourChatAccountProvider(thisAccountId!));
    for (Int64 sessionId in accountData.sessions) {
      core_session.OurChatSession sessionNotifier = ref.read(
        core_session.ourChatSessionProvider(sessionId).notifier,
      );
      await sessionNotifier.getSessionInfo();
      final sessionData = ref.read(
        core_session.ourChatSessionProvider(sessionId),
      );
      if ((sessionData.description.toLowerCase().contains(searchKeyword) ||
              sessionData.name.toLowerCase().contains(searchKeyword) ||
              sessionNotifier.getDisplayName().toLowerCase().contains(
                searchKeyword,
              )) &&
          !matchSessions.contains(sessionId)) {
        matchSessions.add(sessionId);
      }
    }
    return matchSessions;
  }
}
