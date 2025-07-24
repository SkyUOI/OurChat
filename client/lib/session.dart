import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/session.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'dart:async';
import 'package:ourchat/core/ui.dart';
import 'package:fixnum/fixnum.dart';

const emptyTab = 0;
const sessionTab = 1;
const userTab = 2;

class SessionState extends ChangeNotifier {
  int tabIndex = emptyTab;
  OurchatSession? currentSession;
  Int64? currentUserId;
  String tabTitle = "";
  List<BundleMsgs> currentSessionRecords = [];
  List<OurchatSession> sessionsList = [];
  Map<OurchatSession, BundleMsgs> sessionRecentMsg = {};
  void update() {
    notifyListeners();
  }

  void receiveMsg(BundleMsgs eventObj) async {
    await eventObj.sender!.getAccountInfo();
    sessionRecentMsg[eventObj.session!] = eventObj;
    if (currentSession == eventObj.session) {
      currentSessionRecords.insert(0, eventObj);
    }
    update();
  }

  void getSessions(OurchatAppState ourchatAppState) async {
    sessionsList = [];
    for (int i = 0; i < ourchatAppState.thisAccount!.sessions.length; i++) {
      OurchatSession session = OurchatSession(
          ourchatAppState, ourchatAppState.thisAccount!.sessions[i]);
      await session.getSessionInfo();
      List<BundleMsgs> record = await ourchatAppState.eventSystem!
          .getSessionEvent(ourchatAppState, session, num: 1);
      sessionsList.add(session);
      sessionRecentMsg[session] = record[0];
    }
    update();
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
    OurchatAppState appState = context.watch<OurchatAppState>();

    return ChangeNotifierProvider(
      create: (context) => SessionState(),
      builder: (context, child) {
        var sessionState = context.watch<SessionState>();
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
        return LayoutBuilder(
          // 此builder可以在尺寸发生变化时重新构建
          builder: (context, constraints) {
            Widget page = const Placeholder();
            // 匹配不同设备类型
            if (appState.device == mobile) {
              if (sessionState.currentSession != null ||
                  sessionState.currentUserId != null) {
                page = Column(
                  children: [
                    Row(
                      children: [
                        BackButton(
                          onPressed: () {
                            sessionState.tabTitle = "";
                            sessionState.currentUserId = null;
                            sessionState.currentSession = null;
                            sessionState.currentSessionRecords = [];
                            sessionState.update();
                          },
                        ),
                        Text(sessionState.tabTitle,
                            style: TextStyle(fontSize: 20))
                      ],
                    ),
                    Expanded(child: tab)
                  ],
                );
              } else {
                page = SessionList();
              }
            } else if (appState.device == desktop) {
              page = Row(
                children: [
                  Flexible(
                      flex: 1, child: cardWithPadding(const SessionList())),
                  Flexible(
                      flex: 3,
                      child: Column(
                        children: [
                          Text(sessionState.tabTitle,
                              style: TextStyle(fontSize: 30)),
                          Expanded(child: tab)
                        ],
                      )),
                ],
              );
            }
            return page;
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

  Future getAccountInfo(OurchatAppState ourchatAppState, Int64 id) async {
    OurchatAccount account = OurchatAccount(ourchatAppState);
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
      OurchatAppState ourchatAppState, OurchatAccount account) {
    var l10n = AppLocalizations.of(context);
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
                          InputDecoration(label: Text(l10n!.addFriendMessage)),
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
              ElevatedButton(
                  onPressed: () {
                    Navigator.pop(context);
                  },
                  child: Text(l10n.cancel)),
              ElevatedButton(
                  onPressed: () async {
                    formKey.currentState!.save();
                    var stub = OurChatServiceClient(
                        ourchatAppState.server!.channel!,
                        interceptors: [ourchatAppState.server!.interceptor!]);
                    Navigator.pop(context);
                    try {
                      await stub.addFriend(AddFriendRequest(
                          friendId: account.id,
                          displayName: addFriendDisplayName,
                          leaveMessage: addFriendLeaveMessage));
                      if (context.mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                          content:
                              Text(AppLocalizations.of(context)!.succeeded),
                        ));
                      }
                    } on grpc.GrpcError catch (e) {
                      if (context.mounted) {
                        switch (e.code) {
                          case internalStatusCode:
                            // 服务端内部错误
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(
                                  AppLocalizations.of(context)!.serverError),
                            ));
                            break;
                          case unavailableStatusCode:
                            // 服务端维护中
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(AppLocalizations.of(context)!
                                  .serverStatusUnderMaintenance),
                            ));
                            break;
                          case permissionDeniedStatusCode:
                            // 没有权限
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(AppLocalizations.of(context)!
                                  .permissionDenied),
                            ));
                          case alreadyExistsStatusCode:
                            // 好友关系已存在
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(AppLocalizations.of(context)!
                                  .friendAlreadyExists),
                            ));
                          case notFoundStatusCode:
                            // 用户不存在
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(
                                  AppLocalizations.of(context)!.userNotFound),
                            ));
                          default:
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Text(
                                  AppLocalizations.of(context)!.unknownError),
                            ));
                        }
                        ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(l10n.internalError)));
                      }
                    }
                  },
                  child: Text(l10n.send))
            ],
          );
        });
  }

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var sessionState = context.watch<SessionState>();
    var l10n = AppLocalizations.of(context);
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
                  Text(l10n!.loading)
                ],
              ),
            );
          }
          OurchatAccount account = snapshot.data;
          bool isFriend =
              ourchatAppState.thisAccount!.friends.contains(account.id);
          var i10n = AppLocalizations.of(context);
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                SizedBox(width: 100, height: 100, child: Placeholder()),
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
                        userInfoRow(i10n!.displayName, account.displayName!),
                      userInfoRow(i10n!.username, account.username),
                      userInfoRow(i10n.ocid, account.ocid),
                    ],
                  ),
                ),
                if (!isFriend)
                  ElevatedButton(
                      onPressed: () => showAddFriendDialog(
                          context, ourchatAppState, account),
                      child: Text(i10n.addFriend)),
              ],
            ),
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
  bool showSearchResults = false; // 正在显示搜索结果
  bool search = false; // 搜索中
  String searchKeyword = "";
  late OurchatAppState ourchatAppState;
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
    ourchatAppState = context.watch<OurchatAppState>();
    sessionState = context.watch<SessionState>();
    if (!inited) {
      ourchatAppState.eventSystem!.addListener(
          FetchMsgsResponse_RespondEventType.msg, sessionState.receiveMsg);
      sessionState.getSessions(ourchatAppState);
      inited = true;
    }
    var l10n = AppLocalizations.of(context)!;
    return LayoutBuilder(builder: (context, constraints) {
      return Column(
        children: [
          Row(
            children: [
              Expanded(
                  child: TextFormField(
                // 搜索框
                decoration: const InputDecoration(hintText: "Search"),
                onChanged: (value) {
                  setState(() {
                    searchKeyword = value;
                    showSearchResults = value.isNotEmpty;
                    search = false;
                  });
                  _debounceTimer!.cancel();
                  _debounceTimer = Timer(
                      const Duration(seconds: 1),
                      () => setState(() {
                            search = true; // 一秒内没输入，搜索
                          }));
                },
              )),
              IconButton(onPressed: () {}, icon: const Icon(Icons.add)) // 创建会话
            ],
          ),
          if (showSearchResults)
            const Align(alignment: Alignment.centerLeft, child: Text("OCID")),
          if (showSearchResults && search)
            FutureBuilder(
                future: getAccountInfo(ourchatAppState, searchKeyword, context),
                builder: (BuildContext context, AsyncSnapshot snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    // 未完成
                    return CircularProgressIndicator(
                      // 显示加载图标
                      color: Theme.of(context).primaryColor,
                    );
                  }
                  OurchatAccount? account = snapshot.data; // 获取搜索到的账号
                  if (account == null) {
                    // 查无此人
                    return Padding(
                        padding: const EdgeInsets.only(top: 5.0),
                        child:
                            Text(AppLocalizations.of(context)!.userNotFound));
                  }
                  bool isFriend =
                      ourchatAppState.thisAccount!.friends.contains(account.id);
                  return SizedBox(
                      // 显示匹配账号
                      height: 50.0,
                      child: Padding(
                        padding: const EdgeInsets.only(top: 5.0),
                        child: ElevatedButton(
                            style: ButtonStyle(
                                shape: WidgetStateProperty.all(
                                    RoundedRectangleBorder(
                                        borderRadius:
                                            BorderRadius.circular(10.0)))),
                            onPressed: () {
                              sessionState.currentUserId = account.id;
                              sessionState.tabIndex = userTab;
                              sessionState.tabTitle =
                                  AppLocalizations.of(context)!.userInfo;
                              sessionState.update();
                            },
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                              children: [
                                const SizedBox(
                                    width: 40.0,
                                    height: 40.0,
                                    child: Placeholder()),
                                if (isFriend)
                                  Text(
                                      "${account.displayName} (${account.username})")
                                else
                                  Text(account.username)
                              ],
                            )),
                      ));
                }),
          if (showSearchResults) const Divider(),
          if (showSearchResults)
            const Align(
                alignment: Alignment.centerLeft, child: Text("Session Id")),
          if (showSearchResults)
            SizedBox(
                height: 50.0,
                child: Padding(
                  padding: const EdgeInsets.only(top: 5.0),
                  child: ElevatedButton(
                      style: ButtonStyle(
                          shape: WidgetStateProperty.all(RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(10.0)))),
                      onPressed: () {},
                      child: const Placeholder()),
                )),
          if (showSearchResults) const Divider(),
          if (showSearchResults)
            const Align(
              alignment: Alignment.centerLeft,
              child: Text("Others"),
            ),
          Expanded(
            child: ListView.builder(
              itemBuilder: (context, index) {
                OurchatSession currentSession =
                    sessionState.sessionsList[index];
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
                            sessionState.tabIndex = sessionTab;
                            sessionState.tabTitle = (currentSession.name == ""
                                ? l10n.newSession
                                : currentSession.name);
                            sessionState.update();
                            sessionState.currentSessionRecords =
                                await ourchatAppState.eventSystem!
                                    .getSessionEvent(
                                        ourchatAppState, currentSession);
                            sessionState.update();
                          },
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.start,
                            children: [
                              SizedBox(
                                height: 40,
                                width: 40,
                                child: Placeholder(),
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
                                            child: Text(
                                              (currentSession.name == ""
                                                  ? l10n.newSession
                                                  : currentSession.name),
                                              style: TextStyle(
                                                  fontSize: 20,
                                                  color: Colors.black),
                                            )),
                                        Align(
                                          alignment: Alignment.centerLeft,
                                          child: Text(
                                            "${sessionState.sessionRecentMsg[currentSession]!.sender!.username}: ${sessionState.sessionRecentMsg[currentSession]!.msgs[0].text}",
                                            style:
                                                TextStyle(color: Colors.grey),
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

  Future getSessionInfo(
      OurchatAppState ourchatAppState, Int64 sessionId) async {
    OurchatSession session = OurchatSession(ourchatAppState, sessionId);
    await session.getSessionInfo();
    return session;
  }

  Future getAccountInfo(
      OurchatAppState ourchatAppState, String ocid, context) async {
    BasicServiceClient stub =
        BasicServiceClient(ourchatAppState.server!.channel!, interceptors: []);
    try {
      var res = await stub.getId(GetIdRequest(ocid: ocid));
      OurchatAccount account = OurchatAccount(ourchatAppState);
      account.id = res.id;
      account.recreateStub();
      await account.getAccountInfo();
      return account;
    } on grpc.GrpcError catch (e) {
      if (context.mounted) {
        switch (e.code) {
          case internalStatusCode: // 服务端内部错误
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                content: Text(AppLocalizations.of(context)!.serverError)));
            break;
          case unavailableStatusCode: // 服务端维护中
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
              content: Text(
                  AppLocalizations.of(context)!.serverStatusUnderMaintenance),
            ));
            break;
          case permissionDeniedStatusCode: // 权限不足，理论上不会出现
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
              content: Text(AppLocalizations.of(context)!.permissionDenied),
            ));
            break;
          case invalidArgumentStatusCode: // 请求字段错误，理论上不会出现
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
              content: Text(AppLocalizations.of(context)!.internalError),
            ));
            break;
          case notFoundStatusCode: // 查无此人
            break;
          default: // 未知错误
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
              content: Text(AppLocalizations.of(context)!.unknownError),
            ));
            break;
        }
      }
    }
    return null;
  }
}

class SessionTab extends StatelessWidget {
  const SessionTab({super.key});

  @override
  Widget build(BuildContext context) {
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    SessionState sessionState = context.watch<SessionState>();
    var l10n = AppLocalizations.of(context)!;
    var key = GlobalKey<FormState>();
    TextEditingController controller = TextEditingController();
    return Form(
      key: key,
      child: Stack(
        children: [
          Column(
            mainAxisSize: MainAxisSize.max,
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Flexible(
                  flex: 10,
                  child: cardWithPadding(const SessionRecord())), //聊天记录
              Flexible(
                // 输入框
                flex: 2,
                child: cardWithPadding(Align(
                  alignment: Alignment.bottomCenter,
                  child: SingleChildScrollView(
                    child: TextFormField(
                      decoration: InputDecoration(hintText: "Type here..."),
                      maxLines: null,
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return l10n.cantBeEmpty;
                        }
                        return null;
                      },
                      onSaved: (value) async {
                        BundleMsgs bundleMsgs = BundleMsgs(ourchatAppState,
                            sender: ourchatAppState.thisAccount!,
                            msgs: [
                              OneMessage(messageType: textMsg, text: value)
                            ]);
                        controller.text = "";
                        var res =
                            await bundleMsgs.send(sessionState.currentSession!);
                        // TODO: deal with error
                      },
                      controller: controller,
                    ),
                  ),
                )),
              ),
            ],
          ),
          Align(
            alignment: Alignment.bottomRight,
            child: FloatingActionButton.extended(
                onPressed: () {
                  if (key.currentState!.validate()) {
                    key.currentState!.save();
                  }
                },
                label: Text(l10n.send)),
          )
        ],
      ),
    );
  }
}

class SessionRecord extends StatefulWidget {
  const SessionRecord({super.key});

  @override
  State<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends State<SessionRecord> {
  @override
  Widget build(BuildContext context) {
    SessionState sessionState = context.watch<SessionState>();
    return ListView.builder(
      itemBuilder: (context, index) {
        String username =
            sessionState.currentSessionRecords[index].sender!.username;
        List<Widget> messages = [];
        for (int i = 0;
            i < sessionState.currentSessionRecords[index].msgs.length;
            i++) {
          messages.add(
              Text(sessionState.currentSessionRecords[index].msgs[i].text!));
        }
        bool isMe = sessionState.currentSessionRecords[index].sender!.isMe;
        Widget avatar = Image.asset("assets/images/logo.png", height: 30.0);
        Widget message = Column(
          crossAxisAlignment:
              (isMe ? CrossAxisAlignment.end : CrossAxisAlignment.start),
          children: [
            Text(username),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(8.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: messages,
                ),
              ),
            ),
          ],
        );
        return Container(
          margin: const EdgeInsets.all(5.0),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: // 根据是否为本账号的发言决定左右对齐
                (isMe ? MainAxisAlignment.end : MainAxisAlignment.start),
            children: [(isMe ? message : avatar), (isMe ? avatar : message)],
          ),
        );
      },
      itemCount: sessionState.currentSessionRecords.length,
      reverse: true,
    );
  }
}
