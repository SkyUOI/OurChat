import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
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
  int? currentSessionId;
  Int64? currentUserId;
  String tabTitle = "";
  void update() {
    notifyListeners();
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
            tab = SessionWidget();
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
              if (sessionState.currentSessionId != null ||
                  sessionState.currentUserId != null) {
                page = Column(
                  children: [
                    Row(
                      children: [
                        BackButton(
                          onPressed: () {
                            sessionState.tabTitle = "";
                            sessionState.currentUserId = null;
                            sessionState.currentSessionId = null;
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
    account.token = ourchatAppState.thisAccount!.token;
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
    var i10n = AppLocalizations.of(context);
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
                          InputDecoration(label: Text(i10n!.addFriendMessage)),
                      onSaved: (newValue) {
                        addFriendLeaveMessage = newValue!;
                      },
                    ),
                    TextFormField(
                      decoration:
                          InputDecoration(label: Text(i10n.displayName)),
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
                  child: Text(i10n.cancel)),
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
                        ScaffoldMessenger.of(context)
                            .showSnackBar(SnackBar(content: Text("ERROR")));
                      }
                    }
                  },
                  child: Text(i10n.send))
            ],
          );
        });
  }

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    var sessionState = context.watch<SessionState>();
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
                  Text("加载中...")
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

  @override
  void dispose() {
    _debounceTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    SessionState sessionState = context.watch<SessionState>();
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
                return SizedBox(
                    height: 50.0,
                    child: Padding(
                      padding: const EdgeInsets.only(top: 10.0),
                      child: ElevatedButton(
                          style: ButtonStyle(
                              shape: WidgetStateProperty.all(
                                  RoundedRectangleBorder(
                                      borderRadius:
                                          BorderRadius.circular(10.0)))),
                          onPressed: () {},
                          child: const Placeholder()),
                    ));
              },
              itemCount: 10,
            ),
          )
        ],
      );
    });
  }

  Future getAccountInfo(
      OurchatAppState ourchatAppState, String ocid, context) async {
    BasicServiceClient stub =
        BasicServiceClient(ourchatAppState.server!.channel!, interceptors: []);
    try {
      var res = await stub.getId(GetIdRequest(ocid: ocid));
      OurchatAccount account = OurchatAccount(ourchatAppState);
      account.id = res.id;
      account.token = ourchatAppState.thisAccount!.token;
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

class SessionWidget extends StatelessWidget {
  const SessionWidget({super.key});

  @override
  Widget build(BuildContext context) {
    SessionState sessionState = context.watch<SessionState>();
    sessionState.tabTitle = "Title";
    return Column(
      mainAxisSize: MainAxisSize.max,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Flexible(
            flex: 10, child: cardWithPadding(const SessionRecord())), //聊天记录
        Flexible(
          // 输入框
          flex: 2,
          child: cardWithPadding(const Align(
            alignment: Alignment.bottomCenter,
            child: SingleChildScrollView(
              child: TextField(
                decoration: InputDecoration(hintText: "Type here..."),
                maxLines: null,
              ),
            ),
          )),
        ),
      ],
    );
  }
}

class SessionRecord extends StatefulWidget {
  const SessionRecord({super.key});

  @override
  State<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends State<SessionRecord> {
  List<List> records = [
    [
      "User1",
      [const Text("Message1"), const Text("Message1_newLine")],
      true,
    ], // username messages isMe
    [
      "User2",
      [const Text("Message2")],
      false,
    ],
    [
      "User3",
      [const Text("Message3")],
      false,
    ],
    [
      "User4",
      [const Text("Message4")],
      false,
    ],
    [
      "User5",
      [const Text("Message5")],
      false,
    ],
  ];
  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemBuilder: (context, index) {
        String username = records[index][0];
        List<Widget> messages = records[index][1];
        bool isMe = records[index][2];
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
      itemCount: records.length,
    );
  }
}
