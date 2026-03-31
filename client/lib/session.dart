import 'dart:math';
import 'dart:typed_data';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:image_picker/image_picker.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/session.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
import 'package:ourchat/service/ourchat/friends/set_friend_info/v1/set_friend_info.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/delete_session/v1/delete_session.pb.dart';
import 'package:ourchat/service/ourchat/session/leave_session/v1/leave_session.pb.dart';
import 'package:ourchat/service/ourchat/session/new_session/v1/session.pb.dart';
import 'package:ourchat/service/ourchat/session/set_session_info/v1/set_session_info.pb.dart';
import 'dart:async';
import 'package:ourchat/core/ui.dart';
import 'package:fixnum/fixnum.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:mime/mime.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part "session.freezed.dart";
part "session.g.dart";

enum TabType { empty, session, user }

@freezed
abstract class SessionState with _$SessionState {
  factory SessionState({
    @Default(TabType.empty) TabType tabIndex,
    Int64? currentSessionId,
    Int64? currentUserId,
    @Default("") String tabTitle,
    @Default([]) List<UserMsg> currentSessionRecords,
    @Default([]) List<Int64> sessionsList,
    @Default({}) Map<Int64, UserMsg> sessionLatestMsg,
    @Default({}) Map<String, Uint8List> cacheFiles,
    @Default({}) Map<String, String> cacheFilesContentType,
    @Default({}) Map<String, bool> cacheFilesSendRaw,
    @Default([]) List<String> needUploadFiles,
    @Default(1) int recordLoadCnt,
    @Default(0) double lastPixels,
    @Default(false) bool sessionsLoading,
  }) = _SessionState;
}

@riverpod
class InputText extends _$InputText {
  @override
  String build() {
    return "";
  }

  void setText(String text) {
    state = text;
  }
}

@riverpod
class SessionNotifier extends _$SessionNotifier {
  bool _disposed = false;

  @override
  SessionState build() {
    _disposed = false;
    ref.onDispose(() => _disposed = true);
    return SessionState();
  }

  void receiveMsg(UserMsg eventObj) {
    final latestMsg = Map<Int64, UserMsg>.from(state.sessionLatestMsg);
    latestMsg[eventObj.sessionId!] = eventObj;
    if (state.currentSessionId == eventObj.sessionId) {
      state = state.copyWith(
        sessionLatestMsg: latestMsg,
        currentSessionRecords: [eventObj, ...state.currentSessionRecords],
      );
    } else {
      state = state.copyWith(sessionLatestMsg: latestMsg);
    }
  }

  Future<void> loadSessions() async {
    state = state.copyWith(sessionsLoading: true);
    final thisAccountId = ref.read(thisAccountIdProvider);
    if (thisAccountId == null) return;
    List<Int64> sessionsList = [];
    Map<Int64, UserMsg> latestMsg = {};
    final accountData = ref.read(ourChatAccountProvider(thisAccountId));
    final eventSystem = ref.read(ourChatEventSystemProvider.notifier);
    for (int i = 0; i < accountData.sessions.length; i++) {
      Int64 sessionId = accountData.sessions[i];
      OurChatSession sessionNotifier = ref.read(
        ourChatSessionProvider(sessionId).notifier,
      );
      await sessionNotifier.getSessionInfo();
      if (_disposed) return;
      List<UserMsg> record = await eventSystem.getSessionEvent(
        sessionId,
        num: 1,
      );
      if (_disposed) return;
      sessionsList.add(sessionId);
      if (record.isNotEmpty) {
        latestMsg[sessionId] = record[0];
      }
    }
    state = state.copyWith(
      sessionsList: sessionsList,
      sessionLatestMsg: latestMsg,
      sessionsLoading: false,
    );
  }

  void openUserTab(Int64 userId, String title) {
    state = state.copyWith(
      currentUserId: userId,
      tabIndex: TabType.user,
      tabTitle: title,
      cacheFiles: {},
      cacheFilesContentType: {},
    );
  }

  void openSessionTab(Int64 sessionId, String title, {List<UserMsg>? records}) {
    state = state.copyWith(
      currentSessionId: sessionId,
      tabIndex: TabType.session,
      tabTitle: title,
      currentSessionRecords: records ?? [],
      cacheFiles: {},
      cacheFilesContentType: {},
      recordLoadCnt: 1,
    );
  }

  void clearTab() {
    state = state.copyWith(
      tabTitle: "",
      currentUserId: null,
      currentSessionId: null,
      currentSessionRecords: [],
    );
  }

  void addRecords(List<UserMsg> records) {
    state = state.copyWith(
      currentSessionRecords: [...records, ...state.currentSessionRecords],
      recordLoadCnt: state.recordLoadCnt + 1,
    );
  }

  void setLastPixels(double pixels) {
    state = state.copyWith(lastPixels: pixels);
  }

  void updateTabTitle(String title) {
    state = state.copyWith(tabTitle: title);
  }

  void resetInputArea() {
    state = state.copyWith(
      needUploadFiles: [],
      cacheFiles: {},
      cacheFilesContentType: {},
    );
  }

  void addNeedUploadFile(String path) {
    state = state.copyWith(needUploadFiles: [...state.needUploadFiles, path]);
  }

  void updateCacheFiles(
    Map<String, Uint8List> files,
    Map<String, String> contentTypes,
    Map<String, bool> sendRaw,
  ) {
    state = state.copyWith(
      cacheFiles: files,
      cacheFilesContentType: contentTypes,
      cacheFilesSendRaw: sendRaw,
    );
  }

  void clearNeedUploadFiles() {
    state = state.copyWith(needUploadFiles: []);
  }

  void switchSendRaw(String uri) {
    Map<String, bool> sendRaw = Map.from(state.cacheFilesSendRaw);
    sendRaw[uri] = !sendRaw[uri]!;
    state = state.copyWith(cacheFilesSendRaw: sendRaw);
  }
}

class Session extends ConsumerStatefulWidget {
  const Session({super.key});

  @override
  ConsumerState<Session> createState() => _SessionWidgetState();
}

class _SessionWidgetState extends ConsumerState<Session> {
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      // 此builder可以在尺寸发生变化时重新构建
      builder: (context, constraints) {
        if (ref.watch(screenModeProvider) == ScreenMode.desktop) {
          return Row(
            children: [
              Flexible(flex: 1, child: cardWithPadding(const SessionList())),
              Flexible(flex: 3, child: TabWidget()),
            ],
          );
        } else {
          return SessionList();
        }
      },
    );
  }
}

class EmptyTab extends StatelessWidget {
  const EmptyTab({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(child: Image.asset("assets/images/logo.png", width: 300));
  }
}

class UserTab extends ConsumerStatefulWidget {
  const UserTab({super.key});

  @override
  ConsumerState<UserTab> createState() => _UserTabState();
}

class _UserTabState extends ConsumerState<UserTab> {
  String addFriendLeaveMessage = "", addFriendDisplayName = "";

  Future<bool> fetchAccountInfo(Int64 id) async {
    final notifier = ref.read(ourChatAccountProvider(id).notifier);
    notifier.recreateStub();
    return await notifier.getAccountInfo();
  }

  TableRow userInfoRow(String field, String value) {
    return TableRow(
      children: [
        TableCell(
          child: Text(
            field,
            style: TextStyle(color: Colors.grey),
            textAlign: TextAlign.right,
          ),
        ),
        TableCell(child: Container()), // Spacer
        TableCell(child: Text(value, textAlign: TextAlign.left)),
      ],
    );
  }

  void showAddFriendDialog(BuildContext context, Int64 accountId) {
    final accountData = ref.read(ourChatAccountProvider(accountId));
    showDialog(
      context: context,
      builder: (context) {
        var formKey = GlobalKey<FormState>();
        return AlertDialog(
          content: Form(
            key: formKey,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextFormField(
                  decoration: InputDecoration(
                    label: Text(l10n.addFriendMessage),
                  ),
                  onSaved: (newValue) {
                    addFriendLeaveMessage = newValue!;
                  },
                ),
                TextFormField(
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) {
                    addFriendDisplayName = newValue!;
                  },
                ),
              ],
            ),
          ),
          actions: [
            ElevatedButton.icon(
              style: AppStyles.defaultButtonStyle,
              icon: Icon(Icons.close),
              onPressed: () {
                Navigator.pop(context);
              },
              label: Text(l10n.cancel),
            ),
            ElevatedButton.icon(
              style: AppStyles.defaultButtonStyle,
              icon: Icon(Icons.send),
              onPressed: () async {
                formKey.currentState!.save();
                var stub = ref.watch(ourChatServerProvider).newStub();
                Navigator.pop(context);
                try {
                  await safeRequest(
                    stub.addFriend,
                    AddFriendRequest(
                      friendId: accountData.id,
                      displayName: addFriendDisplayName,
                      leaveMessage: addFriendLeaveMessage,
                    ),
                    (grpc.GrpcError e) {
                      showResultMessage(
                        e.code,
                        e.message,
                        permissionDeniedStatus: l10n.permissionDenied(
                          l10n.addFriend,
                        ),
                        alreadyExistsStatus: l10n.friendAlreadyExists,
                        notFoundStatus: l10n.notFound(l10n.user),
                      );
                    },
                    rethrowError: true,
                  );
                  showResultMessage(okStatusCode, null);
                } catch (e) {
                  // do nothing
                }
              },
              label: Text(l10n.send),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var sessionState = ref.watch(sessionProvider);
    final userId = sessionState.currentUserId!;
    return FutureBuilder(
      future: fetchAccountInfo(userId),
      builder: (context, snapshot) {
        if (snapshot.connectionState != ConnectionState.done) {
          // 尚未成功获取账号信息
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                CircularProgressIndicator(
                  color: Theme.of(context).primaryColor,
                ),
                Text(l10n.loading),
              ],
            ),
          );
        }
        final accountData = ref.read(ourChatAccountProvider(userId));

        final accountNotifier = ref.read(
          ourChatAccountProvider(userId).notifier,
        );
        final currentAccountData = ref.read(
          ourChatAccountProvider(thisAccountId!),
        );
        bool isFriend = currentAccountData.friends.contains(accountData.id);
        return Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Padding(
                padding: EdgeInsets.all(AppStyles.mediumPadding),
                child: UserAvatar(
                  imageUrl: accountNotifier.avatarUrl(),
                  size: AppStyles.largeAvatarSize,
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(20.0),
                child: Table(
                  columnWidths: {
                    0: FlexColumnWidth(15),
                    1: FlexColumnWidth(1),
                    2: FlexColumnWidth(15),
                  },
                  children: [
                    if (accountData.displayName != null)
                      userInfoRow(l10n.displayName, accountData.displayName!),
                    userInfoRow(l10n.username, accountData.username),
                    userInfoRow(l10n.ocid, accountData.ocid),
                  ],
                ),
              ),
              if (!isFriend)
                ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.person_add),
                  onPressed: () => showAddFriendDialog(context, userId),
                  label: Text(l10n.addFriend),
                ),
              if (isFriend)
                ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.edit),
                  onPressed: () => showSetDisplayNameDialog(context, userId),
                  label: Text(l10n.modify),
                ),
            ],
          ),
        );
      },
    );
  }

  void showSetDisplayNameDialog(BuildContext context, Int64 accountId) {
    final accountData = ref.read(ourChatAccountProvider(accountId));
    showDialog(
      context: context,
      builder: (context) {
        var key = GlobalKey<FormState>();
        return AlertDialog(
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Form(
                key: key,
                child: TextFormField(
                  initialValue: accountData.displayName,
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) async {
                    var stub = ref.watch(ourChatServerProvider).newStub();

                    try {
                      await safeRequest(
                        stub.setFriendInfo,
                        SetFriendInfoRequest(
                          id: accountId,
                          displayName: newValue,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(e.code, e.message);
                        },
                      );

                      showResultMessage(okStatusCode, null);

                      await ref
                          .read(ourChatAccountProvider(accountId).notifier)
                          .getAccountInfo(ignoreCache: true);
                    } catch (e) {
                      // do nothing
                    }

                    if (context.mounted) {
                      Navigator.pop(context);
                    }
                  },
                ),
              ),
            ],
          ),
          actions: [
            IconButton(
              onPressed: () {
                key.currentState!.save();
              },
              icon: Icon(Icons.check),
            ),
            IconButton(
              onPressed: () {
                Navigator.pop(context);
              },
              icon: Icon(Icons.close),
            ),
          ],
        );
      },
    );
  }
}

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
                    // 未完成
                    return CircularProgressIndicator(
                      // 显示加载图标
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<Int64> accountIds = snapshot.data; // 获取搜索到的账号ID
                  if (accountIds.isEmpty) {
                    // NotFound
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
                    // 未完成
                    return CircularProgressIndicator(
                      // 显示加载图标
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<Int64> sessionIds = snapshot.data; // 获取搜索到的会话ID列表
                  if (sessionIds.isEmpty) {
                    // NotFount
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
                          ourChatSessionProvider(sessionId).notifier,
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
                            ourChatSessionProvider(currentSessionId).notifier,
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

      OurChatSession sessionNotifier = ref.read(
        ourChatSessionProvider(sessionId).notifier,
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
      OurChatSession sessionNotifier = ref.read(
        ourChatSessionProvider(sessionId).notifier,
      );
      await sessionNotifier.getSessionInfo();
      final sessionData = ref.read(ourChatSessionProvider(sessionId));
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

class SessionListItem extends StatelessWidget {
  const SessionListItem({
    super.key,
    required this.avatar,
    required this.name,
    required this.onPressed,
  });

  final Function onPressed;
  final Widget avatar;
  final String name;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      // 显示匹配账号
      height: 50.0,
      child: Padding(
        padding: const EdgeInsets.only(top: 5.0),
        child: ElevatedButton(
          style: ButtonStyle(
            shape: WidgetStateProperty.all(
              RoundedRectangleBorder(borderRadius: BorderRadius.circular(10.0)),
            ),
          ),
          onPressed: () => onPressed(),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              SizedBox(width: 40.0, height: 40.0, child: avatar),
              Text(name),
            ],
          ),
        ),
      ),
    );
  }
}

class NewSessionDialog extends ConsumerStatefulWidget {
  final SessionState sessionState;
  const NewSessionDialog({super.key, required this.sessionState});

  @override
  ConsumerState<NewSessionDialog> createState() => _NewSessionDialogState();
}

class _NewSessionDialogState extends ConsumerState<NewSessionDialog> {
  List<Int64> friends = [];
  List<bool> checked = [];
  bool gotFriendList = false, enableE2EE = true;

  void getFriendList() async {
    friends = [];
    final thisAccountId = ref.read(thisAccountIdProvider);
    final currentAccountData = ref.read(ourChatAccountProvider(thisAccountId!));
    for (int i = 0; i < currentAccountData.friends.length; i++) {
      Int64 friendId = currentAccountData.friends[i];
      final friendNotifier = ref.read(
        ourChatAccountProvider(friendId).notifier,
      );
      friendNotifier.recreateStub();
      await friendNotifier.getAccountInfo();
      friends.add(friendId);
    }
    for (int i = 0; i < friends.length; i++) {
      checked.add(false);
    }
    gotFriendList = true;
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    if (!gotFriendList) {
      getFriendList();
    }
    return AlertDialog(
      content: SizedBox(
        height: 450,
        width: 300,
        child: ListView.builder(
          itemBuilder: (context, index) {
            return SizedBox(
              height: 60.0,
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
                  onPressed: () {
                    setState(() {
                      checked[index] = !checked[index];
                    });
                  },
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.start,
                    children: [
                      SizedBox(
                        height: 40,
                        width: 40,
                        child: UserAvatar(
                          imageUrl: ref
                              .read(
                                ourChatAccountProvider(friends[index]).notifier,
                              )
                              .avatarUrl(),
                        ),
                      ),
                      Expanded(
                        child: Padding(
                          padding: EdgeInsets.only(left: 8.0),
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Align(
                                alignment: Alignment.centerLeft,
                                // widthFactor: 1.0,
                                child: Text(
                                  ref
                                      .read(
                                        ourChatAccountProvider(friends[index]),
                                      )
                                      .username,
                                  style: TextStyle(
                                    fontSize: 20,
                                    color: Colors.black,
                                  ),
                                  // overflow: TextOverflow.ellipsis,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                      Checkbox(
                        value: checked[index],
                        onChanged: (v) {
                          setState(() {
                            checked[index] = v!;
                          });
                        },
                      ),
                    ],
                  ),
                ),
              ),
            );
          },
          itemCount: friends.length,
        ),
      ),
      actionsAlignment: MainAxisAlignment.spaceBetween,
      actions: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GestureDetector(
              child: Text(l10n.enableE2EE),
              onTap: () {
                setState(() {
                  enableE2EE = !enableE2EE;
                });
              },
            ),
            Checkbox(
              value: enableE2EE,
              onChanged: (v) {
                setState(() {
                  enableE2EE = v!;
                });
              },
            ),
          ],
        ),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              onPressed: () async {
                List<Int64> members = [];
                for (int i = 0; i < friends.length; i++) {
                  if (checked[i]) {
                    members.add(friends[i]);
                  }
                }
                var stub = ref.watch(ourChatServerProvider).newStub();
                try {
                  await safeRequest(
                    stub.newSession,
                    NewSessionRequest(members: members, e2eeOn: enableE2EE),
                    (grpc.GrpcError e) {
                      showResultMessage(
                        e.code,
                        e.message,
                        notFoundStatus: l10n.notFound(l10n.user),
                      );
                    },
                    rethrowError: true,
                  );
                  await ref
                      .read(ourChatAccountProvider(thisAccountId!).notifier)
                      .getAccountInfo(ignoreCache: true);
                  await ref.read(sessionProvider.notifier).loadSessions();
                } catch (e) {
                  return;
                }
                if (context.mounted) {
                  Navigator.pop(context);
                }
              },
              icon: Icon(Icons.check),
            ),
            IconButton(
              onPressed: () {
                Navigator.pop(context);
              },
              icon: Icon(Icons.close),
            ),
          ],
        ),
      ],
    );
  }
}

class SessionTab extends ConsumerStatefulWidget {
  const SessionTab({super.key});

  @override
  ConsumerState<SessionTab> createState() => _SessionTabState();
}

class _SessionTabState extends ConsumerState<SessionTab> {
  TextEditingController controller = TextEditingController();
  GlobalKey<FormState> inputBoxKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    var sessionState = ref.watch(sessionProvider);
    var key = GlobalKey<FormState>();

    return Form(
      key: key,
      child: Column(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Expanded(child: cardWithPadding(const SessionRecord())), //聊天记录
          Row(
            children: [
              Expanded(
                child: SizedBox(
                  height: 100,
                  child: cardWithPadding(
                    Align(
                      alignment: Alignment.bottomCenter,
                      child: SingleChildScrollView(
                        child: TextFormField(
                          key: inputBoxKey,
                          decoration: InputDecoration(hintText: "Type here..."),
                          maxLines: null,
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return l10n.cantBeEmpty;
                            }
                            return null;
                          },
                          onSaved: (value) async {
                            List<String> involvedFiles = [];
                            String text = value!;
                            int index = 0;
                            if (sessionState.needUploadFiles.isNotEmpty) {
                              showResultMessage(
                                okStatusCode,
                                null,
                                okStatus: l10n.uploading,
                              );
                            }

                            for (String path in sessionState.needUploadFiles) {
                              try {
                                if (!sessionState.cacheFiles.containsKey(
                                  path,
                                )) {
                                  showResultMessage(
                                    notFoundStatusCode,
                                    null,
                                    notFoundStatus: l10n.notFound(
                                      "${l10n.image}($path)",
                                    ),
                                  );
                                  continue;
                                }
                                logger.i(
                                  "Uploading file: $path, compress: ${!sessionState.cacheFilesSendRaw[path]!}",
                                );

                                var res = await upload(
                                  ref.watch(ourChatServerProvider),
                                  sessionState.cacheFiles[path]!,
                                  true,
                                  sessionId: sessionState.currentSessionId!,
                                  compress:
                                      !sessionState.cacheFilesSendRaw[path]!,
                                  contentType:
                                      sessionState.cacheFilesContentType[path]!,
                                );

                                String newPath = "IO://$index";
                                text = replaceMarkdownImageUrls(text, (oldUrl) {
                                  if (oldUrl != path) {
                                    return oldUrl;
                                  }
                                  return newPath;
                                });
                                involvedFiles.add(res.key);
                                index += 1;
                              } catch (e) {
                                showResultMessage(
                                  internalStatusCode,
                                  null,
                                  internalStatus: l10n.failTo(l10n.upload),
                                );
                              }
                            }
                            if (sessionState.needUploadFiles.isNotEmpty) {
                              showResultMessage(okStatusCode, null);
                            }
                            var stub = ref
                                .read(ourChatServerProvider)
                                .newStub();
                            try {
                              await safeRequest(
                                stub.sendMsg,
                                SendMsgRequest(
                                  sessionId: sessionState.currentSessionId!,
                                  markdownText: text,
                                  involvedFiles: involvedFiles,
                                  isEncrypted: false,
                                ),
                                (grpc.GrpcError e) {
                                  showResultMessage(
                                    e.code,
                                    e.message,
                                    notFoundStatus: l10n.notFound(l10n.session),
                                    permissionDeniedStatus: l10n
                                        .permissionDenied(l10n.send),
                                  );
                                },
                              );
                            } catch (e) {
                              // do nothing
                            }
                            controller.text = "";
                            ref.read(inputTextProvider.notifier).setText("");
                            ref.read(sessionProvider.notifier).resetInputArea();
                          },
                          onChanged: (value) {
                            ref.read(inputTextProvider.notifier).setText(value);
                          },
                          controller: controller,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              Column(
                mainAxisAlignment: MainAxisAlignment.spaceAround,
                children: [
                  IconButton(
                    onPressed: () async {
                      var picker = ImagePicker();
                      List<XFile> images = await picker.pickMultiImage();
                      for (XFile i in images) {
                        var uri = Uri.parse(i.path);
                        var bytes = await i.readAsBytes();
                        var sendRaw = false;
                        var contentType = lookupMimeType(
                          i.path,
                          headerBytes: List.from(bytes),
                        )!;
                        var newCacheFiles = Map<String, Uint8List>.from(
                          sessionState.cacheFiles,
                        );
                        newCacheFiles[uri.toString()] = bytes;
                        var newSendRaw = Map<String, bool>.from(
                          sessionState.cacheFilesSendRaw,
                        );
                        newSendRaw[uri.toString()] = sendRaw;
                        var newContentTypes = Map<String, String>.from(
                          sessionState.cacheFilesContentType,
                        );
                        newContentTypes[uri.toString()] = contentType;
                        String breakLine =
                            controller.text.isEmpty ||
                                controller.text.endsWith("\n") // 已有换行
                            ? ""
                            : "\n";
                        controller.text =
                            "${controller.text}$breakLine![${i.name}](${uri.toString()})";
                        ref
                            .read(inputTextProvider.notifier)
                            .setText(controller.text);
                        ref
                            .read(sessionProvider.notifier)
                            .updateCacheFiles(
                              newCacheFiles,
                              newContentTypes,
                              newSendRaw,
                            );
                        ref
                            .read(sessionProvider.notifier)
                            .addNeedUploadFile(uri.toString());
                      }
                    },
                    icon: Icon(Icons.add),
                  ),
                  ElevatedButton.icon(
                    style: AppStyles.defaultButtonStyle,
                    onPressed: () {
                      if (key.currentState!.validate()) {
                        key.currentState!.save();
                      }
                    },
                    label: Text(l10n.send),
                    icon: Icon(Icons.send),
                  ),
                ],
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class TabWidget extends ConsumerStatefulWidget {
  const TabWidget({super.key});

  @override
  ConsumerState<TabWidget> createState() => _TabWidgetState();
}

class _TabWidgetState extends ConsumerState<TabWidget> {
  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    SessionState sessionState = ref.watch(sessionProvider);
    Widget tab;
    switch (sessionState.tabIndex) {
      case TabType.session:
        tab = SessionTab();
        break;
      case TabType.user:
        tab = UserTab();
        break;
      default:
        tab = EmptyTab();
        break;
    }
    Widget page = const Placeholder();
    // 匹配不同设备类型
    if (ref.watch(screenModeProvider) == ScreenMode.mobile) {
      page = SafeArea(
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    BackButton(
                      onPressed: () {
                        ref.read(sessionProvider.notifier).clearTab();
                        Navigator.pop(context);
                      },
                    ),
                    Text(sessionState.tabTitle, style: TextStyle(fontSize: 20)),
                  ],
                ),
                if (sessionState.tabIndex == TabType.session)
                  IconButton(
                    onPressed: () => showSetSessionInfoDialog(
                      context,
                      thisAccountId,
                      sessionState,
                    ),
                    icon: Icon(Icons.more_horiz),
                  ),
              ],
            ),
            Expanded(child: tab),
          ],
        ),
      );
    } else if (ref.watch(screenModeProvider) == ScreenMode.desktop) {
      page = Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Expanded(
                child: Center(
                  child: Text(
                    sessionState.tabTitle,
                    style: TextStyle(fontSize: 30),
                  ),
                ),
              ),
              if (sessionState.tabIndex == TabType.session)
                IconButton(
                  onPressed: () => showSetSessionInfoDialog(
                    context,
                    thisAccountId,
                    sessionState,
                  ),
                  icon: Icon(Icons.more_horiz),
                ),
            ],
          ),
          Expanded(child: tab),
        ],
      );
    }
    return Scaffold(body: page);
  }

  void showSetSessionInfoDialog(
    BuildContext context,
    Int64? thisAccountId,
    SessionState sessionState,
  ) {
    final sessionData = ref.read(
      ourChatSessionProvider(sessionState.currentSessionId!),
    );
    String name = sessionData.name, description = sessionData.description;
    var key = GlobalKey<FormState>();

    showDialog(
      context: context,
      builder: (BuildContext context) {
        bool confirmLeave = false;
        bool confirmDelete = false;
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Text(
                sessionData.name.isEmpty ? l10n.newSession : sessionData.name,
              ),
              content: Form(
                key: key,
                child: SizedBox(
                  width: 150,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Row(
                        children: [
                          Padding(
                            padding: const EdgeInsets.only(right: 5.0),
                            child: Text(l10n.sessionId),
                          ),
                          SelectableText(
                            sessionState.currentSessionId.toString(),
                          ),
                        ],
                      ),
                      TextFormField(
                        initialValue: name,
                        decoration: InputDecoration(
                          label: Text(l10n.sessionName),
                        ),
                        onSaved: (newValue) {
                          name = newValue!;
                        },
                      ),
                      TextFormField(
                        initialValue: description,
                        decoration: InputDecoration(
                          label: Text(l10n.description),
                        ),
                        onSaved: (newValue) {
                          description = newValue!;
                        },
                      ),
                    ],
                  ),
                ),
              ),
              actions: [
                if (sessionData.myPermissions.contains(deleteSessionPermission))
                  IconButton(
                    onPressed: () async {
                      if (!confirmDelete) {
                        setState(() {
                          confirmLeave = false;
                          confirmDelete = true;
                        });
                        rootScaffoldMessengerKey.currentState!.showSnackBar(
                          SnackBar(content: Text(l10n.againToConfirm)),
                        );
                        return;
                      }

                      var stub = ref.watch(ourChatServerProvider).newStub();
                      try {
                        safeRequest(
                          stub.deleteSession,
                          DeleteSessionRequest(
                            sessionId: sessionState.currentSessionId!,
                          ),
                          (grpc.GrpcError e) {
                            showResultMessage(
                              e.code,
                              e.message,
                              notFoundStatus: l10n.notFound(l10n.session),
                              permissionDeniedStatus: l10n.permissionDenied(
                                l10n.delete,
                              ),
                            );
                          },
                          rethrowError: true,
                        );
                        Navigator.pop(context);
                        showResultMessage(okStatusCode, null);
                        await ref
                            .read(
                              ourChatAccountProvider(thisAccountId!).notifier,
                            )
                            .getAccountInfo(ignoreCache: true);
                        await ref.read(sessionProvider.notifier).loadSessions();
                      } catch (e) {
                        // do nothing
                      }
                    },
                    icon: Icon(
                      Icons.delete_forever,
                      color: (confirmDelete ? Colors.redAccent : null),
                    ),
                  ),
                IconButton(
                  onPressed: () async {
                    if (!confirmLeave) {
                      setState(() {
                        confirmDelete = false;
                        confirmLeave = true;
                      });
                      rootScaffoldMessengerKey.currentState!.showSnackBar(
                        SnackBar(content: Text(l10n.againToConfirm)),
                      );
                      return;
                    }

                    var stub = ref.watch(ourChatServerProvider).newStub();
                    try {
                      safeRequest(
                        stub.leaveSession,
                        LeaveSessionRequest(
                          sessionId: sessionState.currentSessionId!,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(
                            e.code,
                            e.message,
                            notFoundStatus: l10n.notFound(l10n.session),
                          );
                        },
                      );
                      showResultMessage(okStatusCode, null);
                      // Navigator.pop(context);
                      await ref
                          .read(ourChatAccountProvider(thisAccountId!).notifier)
                          .getAccountInfo(ignoreCache: true);
                      await ref.read(sessionProvider.notifier).loadSessions();
                    } catch (e) {
                      // do nothing
                    }
                  },
                  icon: Icon(
                    Icons.exit_to_app,
                    color: (confirmLeave ? Colors.redAccent : null),
                  ),
                ),
                IconButton(
                  onPressed: () async {
                    key.currentState!.save();
                    var stub = ref.watch(ourChatServerProvider).newStub();

                    try {
                      await safeRequest(
                        stub.setSessionInfo,
                        SetSessionInfoRequest(
                          sessionId: sessionState.currentSessionId!,
                          name: name,
                          description: description,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(
                            e.code,
                            e.message,
                            alreadyExistsStatus: l10n.conflict,
                            permissionDeniedStatus: l10n.permissionDenied(
                              e.message!,
                            ),
                          );
                        },
                        rethrowError: true,
                      );
                      await ref
                          .read(
                            ourChatSessionProvider(
                              sessionState.currentSessionId!,
                            ).notifier,
                          )
                          .getSessionInfo(ignoreCache: true);
                      setState(() {
                        final updatedData = ref.read(
                          ourChatSessionProvider(
                            sessionState.currentSessionId!,
                          ),
                        );
                        ref
                            .read(sessionProvider.notifier)
                            .updateTabTitle(updatedData.name);
                      });
                      showResultMessage(okStatusCode, null);
                    } catch (e) {
                      // do nothing
                    }
                    if (context.mounted) {
                      Navigator.pop(context);
                    }
                  },
                  icon: Icon(Icons.check),
                ),
                IconButton(
                  onPressed: () {
                    Navigator.pop(context);
                  },
                  icon: Icon(Icons.close),
                ),
              ],
            );
          },
        );
      },
    );
  }
}

class SessionRecord extends ConsumerStatefulWidget {
  const SessionRecord({super.key});

  @override
  ConsumerState<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends ConsumerState<SessionRecord> {
  ScrollController scrollController = ScrollController();

  @override
  void initState() {
    scrollController.addListener(onScroll);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    var sessionState = ref.watch(sessionProvider);
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var inputText = ref.watch(inputTextProvider);
    if (sessionState.recordLoadCnt != 1) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        scrollController.jumpTo(sessionState.lastPixels);
      });
    }
    return ListView.builder(
      controller: scrollController,
      itemBuilder: (context, index) {
        if (index == 0) {
          if (inputText.isEmpty) {
            return Container();
          }
          return MessageWidget(
            msg: UserMsg(senderId: thisAccountId, markdownText: inputText),
            opacity: 0.3,
          );
        } else {
          return MessageWidget(
            msg: sessionState.currentSessionRecords[index - 1],
            opacity: 1.0,
          );
        }
      },
      itemCount: sessionState.currentSessionRecords.length + 1,
      reverse: true,
    );
  }

  void onScroll() async {
    if (scrollController.position.maxScrollExtent -
            scrollController.position.pixels <
        300) {
      var sessionState = ref.read(sessionProvider);
      ref
          .read(sessionProvider.notifier)
          .setLastPixels(scrollController.position.pixels);
      List<UserMsg> records = await ref
          .read(ourChatEventSystemProvider.notifier)
          .getSessionEvent(
            sessionState.currentSessionId!,
            offset: 50 * sessionState.recordLoadCnt,
          );
      if (records.isEmpty ||
          sessionState.currentSessionRecords.contains(records.first)) {
        return;
      }
      ref.read(sessionProvider.notifier).addRecords(records);
    }
  }
}

class MessageWidget extends ConsumerStatefulWidget {
  final UserMsg msg;
  final double opacity;
  const MessageWidget({super.key, required this.msg, required this.opacity});

  @override
  ConsumerState<MessageWidget> createState() => _MessageWidgetState();
}

class _MessageWidgetState extends ConsumerState<MessageWidget> {
  @override
  Widget build(BuildContext context) {
    UserMsg msg = widget.msg;
    double opacity = widget.opacity;
    var sessionState = ref.watch(sessionProvider);
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final senderData = msg.senderId != null
        ? ref.read(ourChatAccountProvider(msg.senderId!))
        : null;
    final senderNotifier = msg.senderId != null
        ? ref.read(ourChatAccountProvider(msg.senderId!).notifier)
        : null;
    final dn = senderData?.displayName;
    String name = dn != null && dn.isNotEmpty
        ? dn
        : (senderData?.username ?? "");
    bool isMe = msg.senderId != null && msg.senderId == thisAccountId;
    Widget avatar = UserAvatar(imageUrl: senderNotifier?.avatarUrl() ?? "");
    TextPainter textPainter = TextPainter(
      text: TextSpan(text: MarkdownToText.convert(msg.markdownText, l10n)),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout(
      maxWidth: ref.read(screenModeProvider) == ScreenMode.desktop
          ? 500.0
          : 250.0,
    );
    Widget message = Column(
      crossAxisAlignment: (isMe
          ? CrossAxisAlignment.end
          : CrossAxisAlignment.start),
      children: [
        Text(name),
        ConstrainedBox(
          constraints: BoxConstraints(
            maxWidth:
                textPainter.width +
                (MarkdownToText.containsImage(msg.markdownText) ? 150.0 : 50.0),
          ),
          child: Markdown(
            selectable: true,
            softLineBreak: true,
            data: msg.markdownText,
            onTapLink: (text, href, title) {
              if (href == null) return;
              showDialog(
                context: context,
                builder: (context) {
                  return AlertDialog(
                    title: Text(l10n.areUSure),
                    content: Text(l10n.toExternalWebsite(href)),
                    actions: [
                      IconButton(
                        onPressed: () {
                          Navigator.pop(context);
                          launchUrl(Uri.parse(href));
                        },
                        icon: Icon(Icons.check),
                      ),
                      IconButton(
                        onPressed: () {
                          Navigator.pop(context);
                        },
                        icon: Icon(Icons.close),
                      ),
                    ],
                  );
                },
              );
            },
            imageBuilder: (uri, title, alt) {
              Widget widget = Text(l10n.internalError);
              if (sessionState.cacheFiles.containsKey(uri.toString())) {
                widget = InkWell(
                  onTap: () {
                    ref
                        .read(sessionProvider.notifier)
                        .switchSendRaw(uri.toString());
                  },
                  child: Stack(
                    children: [
                      Image.memory(sessionState.cacheFiles[uri.toString()]!),
                      if (sessionState.cacheFilesSendRaw[uri.toString()]!)
                        Icon(Icons.raw_on)
                      else
                        Icon(Icons.raw_off),
                    ],
                  ),
                );
              }
              try {
                String content = uri.toString().split("://")[1];
                if (uri.scheme[0] == 'i') {
                  if (uri.scheme[1] == 'o') {
                    widget = FutureBuilder(
                      future: getOurChatFile(
                        ref,
                        msg.involvedFiles[int.parse(content)],
                      ),
                      builder: (content, snapshot) {
                        if (snapshot.hasError) {
                          return Text(
                            l10n.failTo("${l10n.load} ${l10n.image}"),
                          );
                        }
                        if (snapshot.connectionState != ConnectionState.done ||
                            snapshot.data == null) {
                          return CircularProgressIndicator(
                            color: Theme.of(context).primaryColor,
                          );
                        }
                        Uint8List fileBytes = snapshot.data as Uint8List;
                        return Image.memory(fileBytes);
                      },
                    );
                  } else if (uri.scheme[1] == 'n') {
                    var path = content.split(",");
                    String url = "${path[0]}://${path.sublist(1).join(',')}";
                    widget = CachedNetworkImage(
                      imageUrl: url,
                      errorWidget: (context, url, error) => Text(
                        l10n.failTo("${l10n.load} ${l10n.image}($url) "),
                      ),
                    );
                  }
                }
              } catch (e) {
                // do nothing
              }
              return widget;
            },
            noScroll: true,
          ),
        ),
      ],
    );
    return Opacity(
      opacity: opacity,
      child: Container(
        margin: const EdgeInsets.all(5.0),
        decoration: BoxDecoration(),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: // 根据是否为本账号的发言决定左右对齐
          (isMe
              ? MainAxisAlignment.end
              : MainAxisAlignment.start),
          children: [(isMe ? message : avatar), (isMe ? avatar : message)],
        ),
      ),
    );
  }
}
