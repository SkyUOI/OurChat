import 'dart:math';

import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/session.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
import 'package:ourchat/service/ourchat/friends/set_friend_info/v1/set_friend_info.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/new_session/v1/session.pb.dart';
import 'package:ourchat/service/ourchat/session/set_session_info/v1/set_session_info.pb.dart';
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
  OurChatSession? currentSession;
  Int64? currentUserId;
  String tabTitle = "";
  List<BundleMsgs> currentSessionRecords = [];
  List<OurChatSession> sessionsList = [];
  Map<OurChatSession, BundleMsgs> sessionRecentMsg = {};
  bool alreadyDispose = false;
  void update() {
    if (alreadyDispose) return;
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

  void getSessions(OurChatAppState ourchatAppState) async {
    sessionsList = [];
    for (int i = 0; i < ourchatAppState.thisAccount!.sessions.length; i++) {
      OurChatSession session = OurChatSession(
          ourchatAppState, ourchatAppState.thisAccount!.sessions[i]);
      await session.getSessionInfo();
      List<BundleMsgs> record = await ourchatAppState.eventSystem!
          .getSessionEvent(ourchatAppState, session, num: 1);
      sessionsList.add(session);
      if (record.isNotEmpty) {
        sessionRecentMsg[session] = record[0];
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
    OurChatAppState appState = context.watch<OurChatAppState>();

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
            if (appState.screenMode == mobile) {
              if (sessionState.currentSession != null ||
                  sessionState.currentUserId != null) {
                page = Column(
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
                                sessionState.update();
                              },
                            ),
                            Text(sessionState.tabTitle,
                                style: TextStyle(fontSize: 20))
                          ],
                        ),
                        if (sessionState.tabIndex == sessionTab)
                          IconButton(
                              onPressed: () => showSetSessionInfoDialog(
                                  context, appState, sessionState),
                              icon: Icon(Icons.more_horiz))
                      ],
                    ),
                    Expanded(child: tab)
                  ],
                );
              } else {
                page = SessionList();
              }
            } else if (appState.screenMode == desktop) {
              page = Row(
                children: [
                  Flexible(
                      flex: 1, child: cardWithPadding(const SessionList())),
                  Flexible(
                      flex: 3,
                      child: Column(
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
                                        context, appState, sessionState),
                                    icon: Icon(Icons.more_horiz))
                            ],
                          ),
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

  void showSetSessionInfoDialog(BuildContext context, OurChatAppState appState,
      SessionState sessionState) {
    String name = sessionState.currentSession!.name,
        description = sessionState.currentSession!.description;
    var l10n = AppLocalizations.of(context)!;

    showDialog(
        context: context,
        builder: (BuildContext context) {
          var key = GlobalKey<FormState>();
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
              IconButton(
                  onPressed: () async {
                    key.currentState!.save();
                    var stub = OurChatServiceClient(appState.server!.channel!,
                        interceptors: [appState.server!.interceptor!]);
                    try {
                      await safeRequest(
                          stub.setSessionInfo,
                          SetSessionInfoRequest(
                              sessionId: sessionState.currentSession!.sessionId,
                              name: name,
                              description: description));
                      await sessionState.currentSession!
                          .getSessionInfo(ignoreCache: true);
                      sessionState.tabTitle = sessionState.currentSession!.name;
                      if (context.mounted) {
                        showResultMessage(context, okStatusCode, null);
                        Navigator.pop(context);
                      }
                    } on grpc.GrpcError catch (e) {
                      if (context.mounted) {
                        showResultMessage(context, e.code, e.message,
                            alreadyExistsStatus: l10n.conflict,
                            permissionDeniedStatus:
                                l10n.permissionDenied(e.message!));
                        Navigator.pop(context);
                      }
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
                              leaveMessage: addFriendLeaveMessage));
                      if (context.mounted) {
                        showResultMessage(context, okStatusCode, null);
                      }
                    } on grpc.GrpcError catch (e) {
                      if (context.mounted) {
                        showResultMessage(context, e.code, e.message,
                            permissionDeniedStatus:
                                l10n.permissionDenied(l10n.addFriend),
                            alreadyExistsStatus: l10n.friendAlreadyExists,
                            notFoundStatus: l10n.notFound(l10n.user));
                      }
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

  void showSetDisplayNameDialog(
      BuildContext context, OurChatAppState appState, OurChatAccount account) {
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
                    var stub = OurChatServiceClient(appState.server!.channel!,
                        interceptors: [appState.server!.interceptor!]);
                    try {
                      await safeRequest(
                          stub.setFriendInfo,
                          SetFriendInfoRequest(
                              id: account.id, displayName: newValue));
                      if (context.mounted) {
                        showResultMessage(context, okStatusCode, null);
                      }
                      await account.getAccountInfo(ignoreCache: true);
                      appState.update();
                    } on grpc.GrpcError catch (e) {
                      if (context.mounted) {
                        showResultMessage(context, e.code, e.message);
                      }
                    } finally {
                      if (context.mounted) {
                        Navigator.pop(context);
                      }
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
        showResultMessage(context, e.code, e.message,
            notFoundStatus: l10n.notFound(l10n.session),
            invalidArgumentStatus: l10n.invalid(l10n.argument));
      }
      inited = true;
    }
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
                              sessionState.update();
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
                        child: Text(l10n.notFound(l10n.user)));
                  }
                  return SizedBox(
                    height: sessionList.length * 50,
                    child: ListView.builder(
                        itemBuilder: (context, index) {
                          OurChatSession session = sessionList[index];
                          return SessionListItem(
                            avatar: Placeholder(),
                            name: session.getDisplayName(l10n),
                            onPressed: () {
                              sessionState.currentSession = session;
                              sessionState.tabIndex = sessionTab;
                              sessionState.tabTitle =
                                  session.getDisplayName(l10n);
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
                  if (sessionState.sessionRecentMsg
                      .containsKey(currentSession)) {
                    recentMsgText =
                        "${sessionState.sessionRecentMsg[currentSession]!.sender!.username}: ${sessionState.sessionRecentMsg[currentSession]!.msgs[0].text}";
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
                              sessionState.tabIndex = sessionTab;
                              sessionState.tabTitle =
                                  currentSession.getDisplayName(l10n);
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
                                              child: Text(
                                                currentSession
                                                    .getDisplayName(l10n),
                                                style: TextStyle(
                                                    fontSize: 20,
                                                    color: Colors.black),
                                              )),
                                          if (sessionState.sessionRecentMsg
                                              .containsKey(currentSession))
                                            Align(
                                              alignment: Alignment.centerLeft,
                                              child: Text(
                                                recentMsgText,
                                                style: TextStyle(
                                                    color: Colors.grey),
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
      var res = await safeRequest(stub.getId, GetIdRequest(ocid: ocid));
      OurChatAccount account = OurChatAccount(ourchatAppState);
      account.id = res.id;
      account.recreateStub();
      await account.getAccountInfo();
      matchAccounts.add(account);
    } on grpc.GrpcError catch (e) {
      if (context.mounted) {
        showResultMessage(context, e.code, e.message,
            // getAccountInfo
            permissionDeniedStatus: l10n.permissionDenied("Get Account Info"),
            invalidArgumentStatus: l10n.internalError,
            notFoundStatus: "");
      }
    }

    // By username/display_name
    for (Int64 friendsId in ourchatAppState.thisAccount!.friends) {
      OurChatAccount account = OurChatAccount(ourchatAppState);
      account.id = friendsId;
      account.recreateStub();
      await account.getAccountInfo();
      if (!matchAccounts.contains(account) &&
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
    var l10n = AppLocalizations.of(context)!;

    if (sessionId != null) {
      // By sessionId
      try {
        OurChatSession session = OurChatSession(appState, sessionId);
        await session.getSessionInfo();
        matchSessions.add(session);
      } on grpc.GrpcError catch (e) {
        if (context.mounted) {
          showResultMessage(context, e.code, e.message, notFoundStatus: "");
        }
      }
    }

    // by name/description
    for (Int64 sessionId in appState.thisAccount!.sessions) {
      OurChatSession session = OurChatSession(ourchatAppState, sessionId);
      await session.getSessionInfo();
      // print(session.name);
      if ((session.description.toLowerCase().contains(searchKeyword) ||
              session.name.toLowerCase().contains(searchKeyword) ||
              session
                  .getDisplayName(l10n)
                  .toLowerCase()
                  .contains(searchKeyword)) &&
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
                                        child: Text(
                                          friends[index].username,
                                          style: TextStyle(
                                              fontSize: 20,
                                              color: Colors.black),
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
                    await safeRequest(
                        stub.newSession,
                        NewSessionRequest(
                            members: members, e2eeOn: enableE2EE));
                    await ourchatAppState!.thisAccount!
                        .getAccountInfo(ignoreCache: true);
                    sessionState.getSessions(ourchatAppState!);
                  } on grpc.GrpcError catch (e) {
                    if (context.mounted) {
                      showResultMessage(context, e.code, e.message,
                          notFoundStatus: l10n.notFound(l10n.user));
                    }
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
  bool inited = false;
  late OurChatAppState ourchatAppState;
  late SessionState sessionState;

  @override
  void dispose() {
    if (ourchatAppState.screenMode == mobile) {
      ourchatAppState.eventSystem!.removeListener(
          FetchMsgsResponse_RespondEventType.msg, sessionState.receiveMsg);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    ourchatAppState = context.watch<OurChatAppState>();
    sessionState = context.watch<SessionState>();
    if (!inited && ourchatAppState.screenMode == mobile) {
      ourchatAppState.eventSystem!.addListener(
          FetchMsgsResponse_RespondEventType.msg, sessionState.receiveMsg);
      sessionState.getSessions(ourchatAppState);
      inited = true;
    }
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
                        try {
                          await bundleMsgs.send(sessionState.currentSession!);
                        } on grpc.GrpcError catch (e) {
                          if (context.mounted) {
                            showResultMessage(context, e.code, e.message,
                                permissionDeniedStatus:
                                    l10n.permissionDenied(l10n.send),
                                notFoundStatus: l10n.notFound(l10n.session));
                          }
                        }
                      },
                      controller: controller,
                    ),
                  ),
                )),
              ),
            ],
          ),
          Positioned(
            right: 20,
            bottom: 20,
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
        String name =
            sessionState.currentSessionRecords[index].sender!.displayName !=
                        null &&
                    sessionState.currentSessionRecords[index].sender!
                        .displayName!.isNotEmpty
                ? sessionState.currentSessionRecords[index].sender!.displayName!
                : sessionState.currentSessionRecords[index].sender!.username;
        List<Widget> messages = [];
        for (int i = 0;
            i < sessionState.currentSessionRecords[index].msgs.length;
            i++) {
          messages.add(
              Text(sessionState.currentSessionRecords[index].msgs[i].text!));
        }
        bool isMe = sessionState.currentSessionRecords[index].sender!.isMe;
        Widget avatar = UserAvatar(
            imageUrl:
                sessionState.currentSessionRecords[index].sender!.avatarUrl());
        Widget message = ConstrainedBox(
          constraints: BoxConstraints(maxWidth: 500.0),
          child: Column(
            crossAxisAlignment:
                (isMe ? CrossAxisAlignment.end : CrossAxisAlignment.start),
            children: [
              Text(name),
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
          ),
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
