import 'dart:math';
import 'dart:typed_data';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
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
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'dart:async';
import 'package:ourchat/core/ui.dart';
import 'package:fixnum/fixnum.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:mime/mime.dart';

const emptyTab = 0;
const sessionTab = 1;
const userTab = 2;

class SessionState extends ChangeNotifier {
  int tabIndex = emptyTab;
  OurChatSession? currentSession;
  Int64? currentUserId;
  String tabTitle = "";
  List<UserMsg> currentSessionRecords = [];
  List<OurChatSession> sessionsList = [];
  Map<OurChatSession, UserMsg> sessionLatestMsg = {};
  bool alreadyDispose = false;
  Map<String, Uint8List> cacheFiles = {};
  Map<String, String> cacheFilesContentType = {};
  Map<String, ValueNotifier<bool>> cacheFilesSendRaw = {};
  List<String> needUploadFiles = [];
  final ValueNotifier<String> inputText = ValueNotifier<String>("");
  int recordLoadCnt = 1;
  double lastPixels = 0;

  void update() {
    if (alreadyDispose) return;
    try {
      notifyListeners();
    } catch (_) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        notifyListeners();
      });
    }
  }

  void receiveMsg(UserMsg eventObj) async {
    await eventObj.sender!.getAccountInfo();
    sessionLatestMsg[eventObj.session!] = eventObj;
    if (currentSession == eventObj.session) {
      currentSessionRecords.insert(0, eventObj);
    }
    update();
  }

  void getSessions(OurChatAppState ourchatAppState) async {
    sessionsList = [];
    for (int i = 0; i < ourchatAppState.thisAccount!.sessions.length; i++) {
      OurChatSession session = OurChatSession(
          ourchatAppState, ourchatAppState.thisAccount!.sessions[i]);
      await session.getSessionInfo();
      List<UserMsg> record = await ourchatAppState.eventSystem!
          .getSessionEvent(ourchatAppState, session, num: 1);
      sessionsList.add(session);
      if (record.isNotEmpty) {
        sessionLatestMsg[session] = record[0];
      }
    }
    if (sessionsList.isNotEmpty) {
      update();
    }
  }

  @override
  void dispose() {
    alreadyDispose = true;
    super.dispose();
  }
}

class Session extends StatefulWidget {
  const Session({super.key});

  @override
  State<Session> createState() => _SessionState();
}

class _SessionState extends State<Session> {
  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => SessionState(),
      builder: (context, child) {
        return LayoutBuilder(
          // 此builder可以在尺寸发生变化时重新构建
          builder: (context, constraints) {
            var ourchatAppState = context.watch<OurChatAppState>();
            if (ourchatAppState.screenMode == desktop) {
              return Row(
                children: [
                  Flexible(
                      flex: 1, child: cardWithPadding(const SessionList())),
                  Flexible(flex: 3, child: TabWidget()),
                ],
              );
            } else {
              return SessionList();
            }
          },
        );
      },
    );
  }
}

class EmptyTab extends StatelessWidget {
  const EmptyTab({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
        child: Image.asset(
      "assets/images/logo.png",
      width: 300,
    ));
  }
}

class UserTab extends StatefulWidget {
  const UserTab({
    super.key,
  });

  @override
  State<UserTab> createState() => _UserTabState();
}

class _UserTabState extends State<UserTab> {
  String addFriendLeaveMessage = "", addFriendDisplayName = "";

  Future getAccountInfo(OurChatAppState ourchatAppState, Int64 id) async {
    OurChatAccount account = OurChatAccount(ourchatAppState);
    account.id = id;
    account.recreateStub();
    await account.getAccountInfo();
    return account;
  }

  TableRow userInfoRow(String field, String value) {
    return TableRow(children: [
      TableCell(
          child: Text(
        field,
        style: TextStyle(color: Colors.grey),
        textAlign: TextAlign.right,
      )),
      TableCell(child: Container()), // Spacer
      TableCell(
          child: Text(
        value,
        textAlign: TextAlign.left,
      ))
    ]);
  }

  void showAddFriendDialog(BuildContext context,
      OurChatAppState ourchatAppState, OurChatAccount account) {
    var l10n = AppLocalizations.of(context)!;
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
                      decoration:
                          InputDecoration(label: Text(l10n.addFriendMessage)),
                      onSaved: (newValue) {
                        addFriendLeaveMessage = newValue!;
                      },
                    ),
                    TextFormField(
                      decoration:
                          InputDecoration(label: Text(l10n.displayName)),
                      onSaved: (newValue) {
                        addFriendDisplayName = newValue!;
                      },
                    )
                  ],
                )),
            actions: [
              ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.close),
                  onPressed: () {
                    Navigator.pop(context);
                  },
                  label: Text(l10n.cancel)),
              ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.send),
                  onPressed: () async {
                    formKey.currentState!.save();
                    var stub = OurChatServiceClient(
                        ourchatAppState.server!.channel!,
                        interceptors: [ourchatAppState.server!.interceptor!]);
                    Navigator.pop(context);
                    try {
                      await safeRequest(
                          stub.addFriend,
                          AddFriendRequest(
                              friendId: account.id,
                              displayName: addFriendDisplayName,
                              leaveMessage: addFriendLeaveMessage),
                          (grpc.GrpcError e) {
                        showResultMessage(ourchatAppState, e.code, e.message,
                            permissionDeniedStatus:
                                l10n.permissionDenied(l10n.addFriend),
                            alreadyExistsStatus: l10n.friendAlreadyExists,
                            notFoundStatus: l10n.notFound(l10n.user));
                      }, rethrowError: true);
                      showResultMessage(ourchatAppState, okStatusCode, null);
                    } catch (e) {
                      // do nothing
                    }
                  },
                  label: Text(l10n.send))
            ],
          );
        });
  }

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var sessionState = context.watch<SessionState>();
    var l10n = AppLocalizations.of(context)!;
    return FutureBuilder(
        future: getAccountInfo(ourchatAppState, sessionState.currentUserId!),
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            // 尚未成功获取账号信息
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  CircularProgressIndicator(
                      color: Theme.of(context).primaryColor),
                  Text(l10n.loading)
                ],
              ),
            );
          }
          OurChatAccount account = snapshot.data;
          bool isFriend =
              ourchatAppState.thisAccount!.friends.contains(account.id);
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Padding(
                  padding: EdgeInsets.all(AppStyles.mediumPadding),
                  child: UserAvatar(
                    imageUrl: account.avatarUrl(),
                    size: AppStyles.largeAvatarSize,
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(20.0),
                  child: Table(
                    columnWidths: {
                      0: FlexColumnWidth(15),
                      1: FlexColumnWidth(1),
                      2: FlexColumnWidth(15)
                    },
                    children: [
                      if (account.displayName != null)
                        userInfoRow(l10n.displayName, account.displayName!),
                      userInfoRow(l10n.username, account.username),
                      userInfoRow(l10n.ocid, account.ocid),
                    ],
                  ),
                ),
                if (!isFriend)
                  ElevatedButton.icon(
                      style: AppStyles.defaultButtonStyle,
                      icon: Icon(Icons.person_add),
                      onPressed: () => showAddFriendDialog(
                          context, ourchatAppState, account),
                      label: Text(l10n.addFriend)),
                if (isFriend)
                  ElevatedButton.icon(
                      style: AppStyles.defaultButtonStyle,
                      icon: Icon(Icons.edit),
                      onPressed: () => showSetDisplayNameDialog(
                          context, ourchatAppState, account),
                      label: Text(l10n.modify))
              ],
            ),
          );
        });
  }

  void showSetDisplayNameDialog(BuildContext context,
      OurChatAppState ourchatAppState, OurChatAccount account) {
    var l10n = AppLocalizations.of(context)!;
    showDialog(
        context: context,
        builder: (context) {
          var key = GlobalKey<FormState>();
          return AlertDialog(
            content: Column(mainAxisSize: MainAxisSize.min, children: [
              Form(
                key: key,
                child: TextFormField(
                  initialValue: account.displayName,
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) async {
                    var stub = OurChatServiceClient(
                        ourchatAppState.server!.channel!,
                        interceptors: [ourchatAppState.server!.interceptor!]);

                    try {
                      await safeRequest(
                          stub.setFriendInfo,
                          SetFriendInfoRequest(
                              id: account.id,
                              displayName: newValue), (grpc.GrpcError e) {
                        showResultMessage(ourchatAppState, e.code, e.message);
                      });

                      showResultMessage(ourchatAppState, okStatusCode, null);

                      await account.getAccountInfo(ignoreCache: true);
                      ourchatAppState.update();
                    } catch (e) {
                      // do nothing
                    }

                    if (context.mounted) {
                      Navigator.pop(context);
                    }
                  },
                ),
              )
            ]),
            actions: [
              IconButton(
                  onPressed: () {
                    key.currentState!.save();
                  },
                  icon: Icon(Icons.check)),
              IconButton(
                  onPressed: () {
                    Navigator.pop(context);
                  },
                  icon: Icon(Icons.close))
            ],
          );
        });
  }
}

class SessionList extends StatefulWidget {
  const SessionList({super.key});

  @override
  State<SessionList> createState() => _SessionListState();
}

class _SessionListState extends State<SessionList> {
  Timer? _debounceTimer = Timer(Duration.zero, () {}); // 搜索timer
  bool search = false; // 搜索中
  String searchKeyword = "";
  late OurChatAppState ourchatAppState;
  late SessionState sessionState;
  bool inited = false;

  @override
  void dispose() {
    _debounceTimer?.cancel();
    ourchatAppState.eventSystem!.removeListener(
        FetchMsgsResponse_RespondEventType.msg, sessionState.receiveMsg);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    ourchatAppState = context.watch<OurChatAppState>();
    sessionState = context.watch<SessionState>();
    var l10n = AppLocalizations.of(context)!;
    if (!inited) {
      ourchatAppState.eventSystem!.addListener(
          FetchMsgsResponse_RespondEventType.msg, sessionState.receiveMsg);
      try {
        sessionState.getSessions(ourchatAppState);
      } on grpc.GrpcError catch (e) {
        showResultMessage(ourchatAppState, e.code, e.message,
            notFoundStatus: l10n.notFound(l10n.session),
            invalidArgumentStatus: l10n.invalid(l10n.argument));
      }
      inited = true;
    }
    var context_ = context;
    return LayoutBuilder(builder: (context, constraints) {
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
                          }));
                },
              )),
              IconButton(
                  onPressed: () {
                    showDialog(
                        context: context,
                        builder: (context) {
                          return NewSessionDialog(
                            sessionState: sessionState,
                          );
                        });
                  },
                  icon: const Icon(Icons.add)) // 创建会话
            ],
          ),
          if (search)
            Align(alignment: Alignment.centerLeft, child: Text(l10n.user)),
          if (search)
            FutureBuilder(
                future: searchAccount(ourchatAppState, searchKeyword, context),
                builder: (BuildContext context, AsyncSnapshot snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    // 未完成
                    return CircularProgressIndicator(
                      // 显示加载图标
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<OurChatAccount> accountList = snapshot.data; // 获取搜索到的账号
                  if (accountList.isEmpty) {
                    // NotFound
                    return Padding(
                        padding: const EdgeInsets.only(top: 5.0),
                        child: Text(l10n.notFound(l10n.user)));
                  }
                  return SizedBox(
                    height: accountList.length * 50,
                    child: ListView.builder(
                        itemBuilder: (context, index) {
                          OurChatAccount account = accountList[index];
                          return SessionListItem(
                            avatar: UserAvatar(
                              imageUrl: account.avatarUrl(),
                            ),
                            name: account.getNameWithDisplayName(),
                            onPressed: () {
                              sessionState.currentUserId = account.id;
                              sessionState.tabIndex = userTab;
                              sessionState.tabTitle = l10n.userInfo;
                              sessionState.cacheFiles = {};
                              sessionState.cacheFilesContentType = {};
                              sessionState.update();
                              if (ourchatAppState.screenMode == mobile) {
                                Navigator.push(
                                    context,
                                    MaterialPageRoute(
                                        builder: (_) =>
                                            ChangeNotifierProvider.value(
                                              value: Provider.of<SessionState>(
                                                  context_),
                                              child: TabWidget(),
                                            )));
                              }
                            },
                          );
                        },
                        itemCount: accountList.length),
                  );
                }),
          if (search) const Divider(),
          if (search)
            Align(alignment: Alignment.centerLeft, child: Text(l10n.session)),
          if (search)
            FutureBuilder(
                future: searchSession(ourchatAppState, searchKeyword, context),
                builder: (context, snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    // 未完成
                    return CircularProgressIndicator(
                      // 显示加载图标
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  List<OurChatSession> sessionList = snapshot.data; // 获取搜索到的会话
                  if (sessionList.isEmpty) {
                    // NotFount
                    return Padding(
                        padding: const EdgeInsets.only(top: 5.0),
                        child: Text(l10n.notFound(l10n.session)));
                  }
                  return SizedBox(
                    height: sessionList.length * 50,
                    child: ListView.builder(
                        itemBuilder: (context, index) {
                          OurChatSession session = sessionList[index];
                          return SessionListItem(
                            avatar: Placeholder(),
                            name: session.getDisplayName(),
                            onPressed: () {
                              sessionState.currentSession = session;
                              sessionState.tabIndex = sessionTab;
                              sessionState.tabTitle = session.getDisplayName();
                              sessionState.cacheFiles = {};
                              sessionState.cacheFilesContentType = {};
                              sessionState.recordLoadCnt = 1;

                              sessionState.update();
                            },
                          );
                        },
                        itemCount: sessionList.length),
                  );
                }),
          if (!search)
            Expanded(
              child: ListView.builder(
                itemBuilder: (context, index) {
                  OurChatSession currentSession =
                      sessionState.sessionsList[index];
                  String recentMsgText = "";
                  if (sessionState.sessionLatestMsg
                      .containsKey(currentSession)) {
                    recentMsgText =
                        "${sessionState.sessionLatestMsg[currentSession]!.sender!.username}: ${MarkdownToText.convert(sessionState.sessionLatestMsg[currentSession]!.markdownText, l10n)}";
                    if (recentMsgText.length > 25) {
                      recentMsgText = recentMsgText.substring(
                          0, min(25, recentMsgText.length));
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
                                        borderRadius:
                                            BorderRadius.circular(10.0)))),
                            onPressed: () async {
                              sessionState.currentSession = currentSession;
                              sessionState.recordLoadCnt = 1;
                              sessionState.tabIndex = sessionTab;
                              sessionState.tabTitle =
                                  currentSession.getDisplayName();
                              if (ourchatAppState.screenMode == mobile) {
                                Navigator.push(
                                    context,
                                    MaterialPageRoute(
                                        builder: (_) =>
                                            ChangeNotifierProvider.value(
                                              value: Provider.of<SessionState>(
                                                  context_),
                                              child: TabWidget(),
                                            )));
                              }
                              sessionState.currentSessionRecords =
                                  await ourchatAppState.eventSystem!
                                      .getSessionEvent(
                                          ourchatAppState, currentSession);
                              sessionState.cacheFiles = {};
                              sessionState.cacheFilesContentType = {};
                              sessionState.update();
                            },
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.start,
                              children: [
                                SizedBox(
                                  height: 40,
                                  width: 40,
                                  child: Image(
                                      image:
                                          AssetImage("assets/images/logo.png")),
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
                                                currentSession.getDisplayName(),
                                                style: TextStyle(
                                                    fontSize: 20,
                                                    color: Theme.of(context)
                                                        .textTheme
                                                        .labelMedium!
                                                        .color),
                                                overflow: TextOverflow.ellipsis,
                                              )),
                                          if (sessionState.sessionLatestMsg
                                              .containsKey(currentSession))
                                            Align(
                                              alignment: Alignment.centerLeft,
                                              widthFactor: 1.0,
                                              child: Text(
                                                recentMsgText,
                                                style: TextStyle(
                                                    color: Colors.grey),
                                                overflow: TextOverflow.ellipsis,
                                              ),
                                            )
                                        ],
                                      )),
                                )
                              ],
                            )),
                      ));
                },
                itemCount: sessionState.sessionsList.length,
              ),
            )
        ],
      );
    });
  }

  Future searchAccount(OurChatAppState ourchatAppState, String ocid,
      BuildContext context) async {
    List<OurChatAccount> matchAccounts = [];
    BasicServiceClient stub =
        BasicServiceClient(ourchatAppState.server!.channel!, interceptors: []);
    var l10n = AppLocalizations.of(context)!;

    // By OCID
    try {
      var res = await safeRequest(stub.getId, GetIdRequest(ocid: ocid),
          (grpc.GrpcError e) {
        showResultMessage(ourchatAppState, e.code, e.message,
            // getAccountInfo
            permissionDeniedStatus: l10n.permissionDenied("Get Account Info"),
            invalidArgumentStatus: l10n.internalError,
            notFoundStatus: "");
      }, rethrowError: true);
      OurChatAccount account = OurChatAccount(ourchatAppState);
      account.id = res.id;
      account.recreateStub();
      if (await account.getAccountInfo()) {
        matchAccounts.add(account);
      }
    } catch (e) {
      // not found
    }

    // By username/display_name

    for (Int64 friendsId in ourchatAppState.thisAccount!.friends) {
      OurChatAccount account = OurChatAccount(ourchatAppState);
      account.id = friendsId;
      account.recreateStub();
      if (await account.getAccountInfo() &&
          !matchAccounts.contains(account) &&
          account
              .getNameWithDisplayName()
              .toLowerCase()
              .contains(searchKeyword)) {
        matchAccounts.add(account);
      }
    }

    return matchAccounts;
  }

  Future searchSession(OurChatAppState appState, String searchKeyword,
      BuildContext context) async {
    Int64? sessionId = Int64.tryParseInt(searchKeyword);
    List<OurChatSession> matchSessions = [];

    if (sessionId != null) {
      // By sessionId

      OurChatSession session = OurChatSession(appState, sessionId);
      try {
        if (await session.getSessionInfo()) {
          matchSessions.add(session);
        }
      } catch (e) {
        logger.e(e);
      }
    }

    // by name/description
    for (Int64 sessionId in appState.thisAccount!.sessions) {
      OurChatSession session = OurChatSession(ourchatAppState, sessionId);
      await session.getSessionInfo();
      // print(session.name);
      if ((session.description.toLowerCase().contains(searchKeyword) ||
              session.name.toLowerCase().contains(searchKeyword) ||
              session.getDisplayName().toLowerCase().contains(searchKeyword)) &&
          !matchSessions.contains(session)) {
        matchSessions.add(session);
      }
    }
    return matchSessions;
  }
}

class SessionListItem extends StatelessWidget {
  const SessionListItem(
      {super.key,
      required this.avatar,
      required this.name,
      required this.onPressed});

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
                  shape: WidgetStateProperty.all(RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10.0)))),
              onPressed: () => onPressed(),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  SizedBox(width: 40.0, height: 40.0, child: avatar),
                  Text(name),
                ],
              )),
        ));
  }
}

class NewSessionDialog extends StatefulWidget {
  final SessionState sessionState;
  const NewSessionDialog({super.key, required this.sessionState});

  @override
  State<NewSessionDialog> createState() => _NewSessionDialogState();
}

class _NewSessionDialogState extends State<NewSessionDialog> {
  List<OurChatAccount> friends = [];
  List<bool> checked = [];
  bool gotFriendList = false, enableE2EE = true;
  OurChatAppState? ourchatAppState;

  void getFriendList() async {
    friends = [];
    for (int i = 0; i < ourchatAppState!.thisAccount!.friends.length; i++) {
      OurChatAccount ourchatAccount = OurChatAccount(ourchatAppState!);
      ourchatAccount.id = ourchatAppState!.thisAccount!.friends[i];
      ourchatAccount.recreateStub();
      await ourchatAccount.getAccountInfo();
      friends.add(ourchatAccount);
    }
    for (int i = 0; i < friends.length; i++) {
      checked.add(false);
    }
    gotFriendList = true;
    ourchatAppState!.update();
  }

  @override
  Widget build(BuildContext context) {
    ourchatAppState = context.watch<OurChatAppState>();
    var sessionState = widget.sessionState;
    var l10n = AppLocalizations.of(context)!;
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
                          shape: WidgetStateProperty.all(RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(10.0)))),
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
                                imageUrl: friends[index].avatarUrl()),
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
                                          friends[index].username,
                                          style: TextStyle(
                                              fontSize: 20,
                                              color: Colors.black),
                                          // overflow: TextOverflow.ellipsis,
                                        )),
                                  ],
                                )),
                          ),
                          Checkbox(
                              value: checked[index],
                              onChanged: (v) {
                                setState(() {
                                  checked[index] = v!;
                                });
                              })
                        ],
                      )),
                ));
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
                }),
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
                      members.add(friends[i].id);
                    }
                  }
                  var stub = OurChatServiceClient(
                      ourchatAppState!.server!.channel!,
                      interceptors: [ourchatAppState!.server!.interceptor!]);
                  try {
                    await safeRequest(stub.newSession,
                        NewSessionRequest(members: members, e2eeOn: enableE2EE),
                        (grpc.GrpcError e) {
                      showResultMessage(ourchatAppState!, e.code, e.message,
                          notFoundStatus: l10n.notFound(l10n.user));
                    }, rethrowError: true);
                    await ourchatAppState!.thisAccount!
                        .getAccountInfo(ignoreCache: true);
                    sessionState.getSessions(ourchatAppState!);
                  } catch (e) {
                    return;
                  }
                  if (context.mounted) {
                    Navigator.pop(context);
                  }
                },
                icon: Icon(Icons.check)),
            IconButton(
                onPressed: () {
                  Navigator.pop(context);
                },
                icon: Icon(Icons.close))
          ],
        )
      ],
    );
  }
}

class SessionTab extends StatefulWidget {
  const SessionTab({super.key});

  @override
  State<SessionTab> createState() => _SessionTabState();
}

class _SessionTabState extends State<SessionTab> {
  late OurChatAppState ourchatAppState;
  late SessionState sessionState;
  TextEditingController controller = TextEditingController();
  GlobalKey<FormState> inputBoxKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    ourchatAppState = context.watch<OurChatAppState>();
    sessionState = context.watch<SessionState>();
    var l10n = AppLocalizations.of(context)!;
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
                  child: cardWithPadding(Align(
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
                              ourchatAppState,
                              okStatusCode,
                              null,
                              okStatus: l10n.uploading,
                            );
                          }

                          for (String path in sessionState.needUploadFiles) {
                            try {
                              if (!sessionState.cacheFiles.containsKey(path)) {
                                showResultMessage(
                                    ourchatAppState, notFoundStatusCode, null,
                                    notFoundStatus:
                                        l10n.notFound("${l10n.image}($path)"));
                                continue;
                              }
                              logger.i(
                                  "Uploading file: $path, compress: ${!sessionState.cacheFilesSendRaw[path]!.value}");

                              var res = await upload(ourchatAppState,
                                  sessionState.cacheFiles[path]!, true,
                                  sessionId:
                                      sessionState.currentSession!.sessionId,
                                  compress: !sessionState
                                      .cacheFilesSendRaw[path]!.value,
                                  contentType: sessionState
                                      .cacheFilesContentType[path]!);

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
                                  ourchatAppState, internalStatusCode, null,
                                  internalStatus: l10n.failTo(l10n.upload));
                            }
                          }
                          if (sessionState.needUploadFiles.isNotEmpty) {
                            showResultMessage(
                              ourchatAppState,
                              okStatusCode,
                              null,
                            );
                          }
                          UserMsg msg = UserMsg(ourchatAppState,
                              sender: ourchatAppState.thisAccount!,
                              markdownText: text,
                              involvedFiles: involvedFiles,
                              session: sessionState.currentSession);
                          controller.text = "";
                          sessionState.inputText.value = "";
                          sessionState.needUploadFiles = [];
                          sessionState.cacheFiles = {};
                          sessionState.cacheFilesContentType = {};
                          await msg.send(sessionState.currentSession!);
                          sessionState.lastPixels = 0;
                        },
                        onChanged: (value) {
                          sessionState.inputText.value = value;
                        },
                        controller: controller,
                      ),
                    ),
                  )),
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
                          sessionState.cacheFiles[uri.toString()] =
                              await i.readAsBytes();
                          sessionState.cacheFilesSendRaw[uri.toString()] =
                              ValueNotifier<bool>(false);
                          sessionState.cacheFilesContentType[uri.toString()] =
                              lookupMimeType(i.path,
                                  headerBytes: List.from(sessionState
                                      .cacheFiles[uri.toString()]!))!;
                          String breakLine = controller.text.isEmpty ||
                                  controller.text.endsWith("\n") // 已有换行
                              ? ""
                              : "\n";
                          controller.text =
                              "${controller.text}$breakLine![${i.name}](${uri.toString()})";
                          sessionState.inputText.value = controller.text;
                        }
                      },
                      icon: Icon(Icons.add)),
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
              )
            ],
          ),
        ],
      ),
    );
  }
}

class TabWidget extends StatefulWidget {
  const TabWidget({super.key});

  @override
  State<TabWidget> createState() => _TabWidgetState();
}

class _TabWidgetState extends State<TabWidget> {
  @override
  Widget build(BuildContext context) {
    OurChatAppState ourchatAppState = context.watch<OurChatAppState>();
    SessionState sessionState = context.watch<SessionState>();
    Widget tab;
    switch (sessionState.tabIndex) {
      case sessionTab:
        tab = SessionTab();
        break;
      case userTab:
        tab = UserTab();
        break;
      default:
        tab = EmptyTab();
        break;
    }
    Widget page = const Placeholder();
    // 匹配不同设备类型
    if (ourchatAppState.screenMode == mobile) {
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
                      sessionState.tabTitle = "";
                      sessionState.currentUserId = null;
                      sessionState.currentSession = null;
                      sessionState.currentSessionRecords = [];
                      Navigator.pop(context);
                    },
                  ),
                  Text(sessionState.tabTitle, style: TextStyle(fontSize: 20))
                ],
              ),
              if (sessionState.tabIndex == sessionTab)
                IconButton(
                    onPressed: () => showSetSessionInfoDialog(
                        context, ourchatAppState, sessionState),
                    icon: Icon(Icons.more_horiz))
            ],
          ),
          Expanded(child: tab)
        ],
      ));
    } else if (ourchatAppState.screenMode == desktop) {
      page = Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Expanded(
                child: Center(
                  child: Text(sessionState.tabTitle,
                      style: TextStyle(fontSize: 30)),
                ),
              ),
              if (sessionState.tabIndex == sessionTab)
                IconButton(
                    onPressed: () => showSetSessionInfoDialog(
                        context, ourchatAppState, sessionState),
                    icon: Icon(Icons.more_horiz))
            ],
          ),
          Expanded(child: tab)
        ],
      );
    }
    return Scaffold(body: page);
  }

  void showSetSessionInfoDialog(BuildContext context,
      OurChatAppState ourchatAppState, SessionState sessionState) {
    String name = sessionState.currentSession!.name,
        description = sessionState.currentSession!.description;
    var l10n = AppLocalizations.of(context)!;
    var key = GlobalKey<FormState>();

    showDialog(
        context: context,
        builder: (BuildContext context) {
          bool confirmLeave = false;
          bool confirmDelete = false;
          return StatefulBuilder(builder: (context, setState) {
            return AlertDialog(
              title: Text(sessionState.currentSession!.name.isEmpty
                  ? l10n.newSession
                  : sessionState.currentSession!.name),
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
                              sessionState.currentSession!.sessionId.toString())
                        ],
                      ),
                      TextFormField(
                        initialValue: name,
                        decoration:
                            InputDecoration(label: Text(l10n.sessionName)),
                        onSaved: (newValue) {
                          name = newValue!;
                        },
                      ),
                      TextFormField(
                        initialValue: description,
                        decoration:
                            InputDecoration(label: Text(l10n.description)),
                        onSaved: (newValue) {
                          description = newValue!;
                        },
                      )
                    ],
                  ),
                ),
              ),
              actions: [
                if (sessionState.currentSession!.myPermissions
                    .contains(deleteSessionPermission))
                  IconButton(
                      onPressed: () async {
                        if (!confirmDelete) {
                          setState(() {
                            confirmLeave = false;
                            confirmDelete = true;
                          });
                          rootScaffoldMessengerKey.currentState!.showSnackBar(
                              SnackBar(content: Text(l10n.againToConfirm)));
                          return;
                        }

                        var stub = OurChatServiceClient(
                            ourchatAppState.server!.channel!,
                            interceptors: [
                              ourchatAppState.server!.interceptor!
                            ]);
                        try {
                          safeRequest(
                              stub.deleteSession,
                              DeleteSessionRequest(
                                  sessionId: sessionState.currentSession!
                                      .sessionId), (grpc.GrpcError e) {
                            showResultMessage(
                                ourchatAppState, e.code, e.message,
                                notFoundStatus: l10n.notFound(l10n.session),
                                permissionDeniedStatus:
                                    l10n.permissionDenied(l10n.delete));
                          }, rethrowError: true);
                          Navigator.pop(context);
                          showResultMessage(
                              ourchatAppState, okStatusCode, null);
                          await ourchatAppState.thisAccount!
                              .getAccountInfo(ignoreCache: true);
                          sessionState.getSessions(ourchatAppState);
                        } catch (e) {
                          // do nothing
                        }
                      },
                      icon: Icon(
                        Icons.delete_forever,
                        color: (confirmDelete ? Colors.redAccent : null),
                      )),
                IconButton(
                    onPressed: () async {
                      if (!confirmLeave) {
                        setState(() {
                          confirmDelete = false;
                          confirmLeave = true;
                        });
                        rootScaffoldMessengerKey.currentState!.showSnackBar(
                            SnackBar(content: Text(l10n.againToConfirm)));
                        return;
                      }

                      var stub = OurChatServiceClient(
                          ourchatAppState.server!.channel!,
                          interceptors: [ourchatAppState.server!.interceptor!]);
                      try {
                        safeRequest(
                            stub.leaveSession,
                            LeaveSessionRequest(
                                sessionId: sessionState.currentSession!
                                    .sessionId), (grpc.GrpcError e) {
                          showResultMessage(ourchatAppState, e.code, e.message,
                              notFoundStatus: l10n.notFound(l10n.session));
                        });
                        showResultMessage(ourchatAppState, okStatusCode, null);
                        // Navigator.pop(context);
                        await ourchatAppState.thisAccount!
                            .getAccountInfo(ignoreCache: true);
                        sessionState.getSessions(ourchatAppState);
                      } catch (e) {
                        // do nothing
                      }
                    },
                    icon: Icon(
                      Icons.exit_to_app,
                      color: (confirmLeave ? Colors.redAccent : null),
                    )),
                IconButton(
                    onPressed: () async {
                      key.currentState!.save();
                      var stub = OurChatServiceClient(
                          ourchatAppState.server!.channel!,
                          interceptors: [ourchatAppState.server!.interceptor!]);

                      try {
                        await safeRequest(
                            stub.setSessionInfo,
                            SetSessionInfoRequest(
                                sessionId:
                                    sessionState.currentSession!.sessionId,
                                name: name,
                                description: description), (grpc.GrpcError e) {
                          showResultMessage(ourchatAppState, e.code, e.message,
                              alreadyExistsStatus: l10n.conflict,
                              permissionDeniedStatus:
                                  l10n.permissionDenied(e.message!));
                        }, rethrowError: true);
                        await sessionState.currentSession!
                            .getSessionInfo(ignoreCache: true);
                        setState(() {
                          sessionState.tabTitle =
                              sessionState.currentSession!.name;
                        });
                        showResultMessage(ourchatAppState, okStatusCode, null);
                      } catch (e) {
                        // do nothing
                      }
                      if (context.mounted) {
                        Navigator.pop(context);
                      }
                    },
                    icon: Icon(Icons.check)),
                IconButton(
                    onPressed: () {
                      Navigator.pop(context);
                    },
                    icon: Icon(Icons.close)),
              ],
            );
          });
        });
  }
}

class SessionRecord extends StatefulWidget {
  const SessionRecord({super.key});

  @override
  State<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends State<SessionRecord> {
  ScrollController scrollController = ScrollController();
  late SessionState sessionState;
  late OurChatAppState ourchatAppState;

  @override
  void initState() {
    scrollController.addListener(onScroll);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    sessionState = context.watch<SessionState>();
    if (sessionState.recordLoadCnt != 1) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        scrollController.jumpTo(sessionState.lastPixels);
      });
    }
    ourchatAppState = context.watch<OurChatAppState>();
    return ListView.builder(
      controller: scrollController,
      itemBuilder: (context, index) {
        if (index == 0) {
          return ValueListenableBuilder(
            valueListenable: sessionState.inputText,
            builder: (context, value, child) {
              if (value.isEmpty) {
                return Container();
              }
              sessionState.needUploadFiles = [];
              return MessageWidget(
                  msg: UserMsg(ourchatAppState,
                      sender: ourchatAppState.thisAccount, markdownText: value),
                  opacity: 0.3);
            },
          );
        } else {
          return MessageWidget(
              msg: sessionState.currentSessionRecords[index - 1], opacity: 1.0);
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
      sessionState.lastPixels = scrollController.position.pixels;
      List<UserMsg> records = await ourchatAppState.eventSystem!
          .getSessionEvent(ourchatAppState, sessionState.currentSession!,
              offset: 50 * sessionState.recordLoadCnt);
      if (records.isEmpty ||
          sessionState.currentSessionRecords.contains(records.first)) {
        return;
      }
      sessionState.currentSessionRecords.addAll(records);
      sessionState.recordLoadCnt++;
      sessionState.update();
    }
  }
}

class MessageWidget extends StatefulWidget {
  final UserMsg msg;
  final double opacity;
  const MessageWidget({super.key, required this.msg, required this.opacity});

  @override
  State<MessageWidget> createState() => _MessageWidgetState();
}

class _MessageWidgetState extends State<MessageWidget> {
  @override
  Widget build(BuildContext context) {
    UserMsg msg = widget.msg;
    double opacity = widget.opacity;
    var ourchatAppState = context.watch<OurChatAppState>();
    var sessionState = context.watch<SessionState>();
    String name =
        msg.sender!.displayName != null && msg.sender!.displayName!.isNotEmpty
            ? msg.sender!.displayName!
            : msg.sender!.username;
    bool isMe = msg.sender!.isMe;
    Widget avatar = UserAvatar(imageUrl: msg.sender!.avatarUrl());
    TextPainter textPainter = TextPainter(
        text: TextSpan(
            text:
                MarkdownToText.convert(msg.markdownText, ourchatAppState.l10n)),
        textDirection: TextDirection.ltr);
    textPainter.layout(
        maxWidth: ourchatAppState.screenMode == desktop ? 500.0 : 250.0);
    Widget message = Column(
      crossAxisAlignment:
          (isMe ? CrossAxisAlignment.end : CrossAxisAlignment.start),
      children: [
        Text(name),
        ConstrainedBox(
          constraints: BoxConstraints(
              maxWidth: textPainter.width +
                  (MarkdownToText.containsImage(msg.markdownText)
                      ? 150.0
                      : 50.0)),
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
                      title: Text(ourchatAppState.l10n.areUSure),
                      content:
                          Text(ourchatAppState.l10n.toExternalWebsite(href)),
                      actions: [
                        IconButton(
                            onPressed: () {
                              Navigator.pop(context);
                              launchUrl(Uri.parse(href));
                            },
                            icon: Icon(Icons.check)),
                        IconButton(
                            onPressed: () {
                              Navigator.pop(context);
                            },
                            icon: Icon(Icons.close)),
                      ],
                    );
                  });
            },
            imageBuilder: (uri, title, alt) {
              Widget widget = Text(ourchatAppState.l10n.internalError);
              if (sessionState.cacheFiles.containsKey(uri.toString())) {
                widget = InkWell(
                  onTap: () {
                    sessionState.cacheFilesSendRaw[uri.toString()]!.value =
                        !sessionState.cacheFilesSendRaw[uri.toString()]!.value;
                  },
                  child: ValueListenableBuilder(
                      valueListenable:
                          sessionState.cacheFilesSendRaw[uri.toString()]!,
                      builder: (context, value, child) {
                        return Stack(
                          children: [
                            Image.memory(
                                sessionState.cacheFiles[uri.toString()]!),
                            if (value)
                              Icon(Icons.raw_on)
                            else
                              Icon(Icons.raw_off)
                          ],
                        );
                      }),
                );
                sessionState.needUploadFiles.add(uri.toString());
              }
              try {
                String content = uri.toString().split("://")[1];
                if (uri.scheme[0] == 'i') {
                  if (uri.scheme[1] == 'o') {
                    widget = FutureBuilder(
                        future: getOurChatFile(ourchatAppState,
                            msg.involvedFiles[int.parse(content)]),
                        builder: (content, snapshot) {
                          if (snapshot.hasError) {
                            return Text(ourchatAppState.l10n.failTo(
                                "${ourchatAppState.l10n.load} ${ourchatAppState.l10n.image}"));
                          }
                          if (snapshot.connectionState !=
                                  ConnectionState.done ||
                              snapshot.data == null) {
                            return CircularProgressIndicator(
                              color: Theme.of(context).primaryColor,
                            );
                          }
                          Uint8List fileBytes = snapshot.data as Uint8List;
                          return Image.memory(fileBytes);
                        });
                  } else if (uri.scheme[1] == 'n') {
                    var path = content.split(",");
                    String url = "${path[0]}://${path.sublist(1).join(',')}";
                    widget = CachedNetworkImage(
                      imageUrl: url,
                      errorWidget: (context, url, error) => Text(
                          ourchatAppState.l10n.failTo(
                              "${ourchatAppState.l10n.load} ${ourchatAppState.l10n.image}($url) ")),
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
        )
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
              (isMe ? MainAxisAlignment.end : MainAxisAlignment.start),
          children: [(isMe ? message : avatar), (isMe ? avatar : message)],
        ),
      ),
    );
  }
}
