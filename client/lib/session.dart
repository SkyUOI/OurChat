import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/ourchat/ourchat_account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:provider/provider.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'dart:async';
import 'package:ourchat/ourchat/ourchat_ui.dart';

class Session extends StatefulWidget {
  const Session({super.key});

  @override
  State<Session> createState() => _SessionState();
}

class _SessionState extends State<Session> {
  int currentSession = -1;
  @override
  Widget build(BuildContext context) {
    OurchatAppState appState = context.watch<OurchatAppState>();
    return LayoutBuilder(
      // 此builder可以在尺寸发生变化时重新构建
      builder: (context, constraints) {
        Widget page = const Placeholder();
        // 匹配不同设备类型
        if (appState.device == mobile) {
          page = SessionList();
        } else if (appState.device == desktop) {
          page = Row(
            children: [
              Flexible(flex: 1, child: cardWithPadding(const SessionList())),
              const Flexible(flex: 3, child: SessionWidget()),
            ],
          );
        }
        return page;
      },
    );
  }
}

class SessionList extends StatefulWidget {
  const SessionList({super.key});

  @override
  State<SessionList> createState() => _SessionListState();
}

class _SessionListState extends State<SessionList> {
  var hoverIndex = -1; // 当前选中的session
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
                            onPressed: () {},
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                              children: [
                                const SizedBox(
                                    width: 40.0,
                                    height: 40.0,
                                    child: Placeholder()),
                                Text(account.displayName!)
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
                          onPressed: () {
                            setState(() {
                              hoverIndex = index;
                              Navigator.push(context,
                                  MaterialPageRoute(builder: (context) {
                                return Placeholder();
                              }));
                            });
                          },
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
      OurchatAccount account = OurchatAccount(ourchatAppState.server!);
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
    OurchatAppState appState = context.watch<OurchatAppState>();
    Widget sessionTitle = const Text("Session", style: TextStyle(fontSize: 30));
    return Column(
      mainAxisSize: MainAxisSize.max,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Flexible(
          flex: 1,
          // 若设备为移动端,显示一个返回按钮
          child: (appState.device == mobile
              ? Row(
                  children: [
                    BackButton(
                      onPressed: () {
                        Navigator.pop(context);
                      },
                    ),
                    sessionTitle,
                  ],
                )
              : Align(alignment: Alignment.center, child: sessionTitle)),
        ),
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
