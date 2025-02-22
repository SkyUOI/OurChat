import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/ourchat/ourchat_account.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:provider/provider.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'dart:async';

class SessionState extends ChangeNotifier {
  int currentSession = -1;

  void setCurrentSession(int index) {
    currentSession = index;
    notifyListeners();
  }
}

class Session extends StatelessWidget {
  const Session({super.key});

  @override
  Widget build(BuildContext context) {
    OurchatAppState appState = context.watch<OurchatAppState>();
    return Scaffold(
      body: ChangeNotifierProvider(
        create: (_) => SessionState(),
        child: LayoutBuilder(
          builder: (context, constraints) {
            SessionState homeState = context.watch<SessionState>();
            Widget page = const Placeholder();
            if (appState.device == mobile) {
              page =
                  (homeState.currentSession == -1
                      ? const SessionList()
                      : const SessionWidget());
            } else if (appState.device == desktop) {
              page = const Row(
                children: [
                  Flexible(flex: 1, child: SessionList()),
                  Flexible(flex: 3, child: SessionWidget()),
                ],
              );
            }

            return page;
          },
        ),
      ),
    );
  }
}

class SessionList extends StatefulWidget {
  const SessionList({super.key});

  @override
  State<SessionList> createState() => _SessionListState();
}

class _SessionListState extends State<SessionList> {
  List<Map<String, String>> sessions = [
    // {"name": "username1", "image": "assets/images/logo.png"},
    // {"name": "username2", "image": "assets/images/logo.png"},
    // {"name": "username3", "image": "assets/images/logo.png"},
    // {"name": "username4", "image": "assets/images/logo.png"},
    // {"name": "username5", "image": "assets/images/logo.png"},
    // {"name": "username6", "image": "assets/images/logo.png"},
    // {"name": "username7", "image": "assets/images/logo.png"},
    // {"name": "username8", "image": "assets/images/logo.png"},
    // {"name": "username9", "image": "assets/images/logo.png"},
    // {"name": "username10", "image": "assets/images/logo.png"}
  ];
  var hoverIndex = -1;
  Timer? _debounceTimer = Timer(Duration.zero, () {});
  bool showSearchResults = false, search = false;
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
                            search = true;
                          }));
                },
              )),
              IconButton(onPressed: () {}, icon: const Icon(Icons.add))
            ],
          ),
          if (showSearchResults)
            const Align(alignment: Alignment.centerLeft, child: Text("OCID")),
          if (showSearchResults && search)
            FutureBuilder(
                future: getAccountInfo(ourchatAppState, searchKeyword),
                builder: (BuildContext context, AsyncSnapshot snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    return Container();
                  }
                  OurchatAccount? account = snapshot.data;
                  if (account == null) {
                    return Padding(
                        padding: const EdgeInsets.only(top: 5.0),
                        child:
                            Text(AppLocalizations.of(context)!.userNotFound));
                  }
                  return SizedBox(
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

  Future getAccountInfo(OurchatAppState ourchatAppState, String ocid) async {
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
          // TODO: deal with error
          case internalStatusCode:
            break;
          case unavailableStatusCode:
            break;
          case notFoundStatusCode:
            break;
          default:
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
    SessionState homeState = context.watch<SessionState>();
    Widget sessionTitle = const Text("Session", style: TextStyle(fontSize: 30));
    return Column(
      mainAxisSize: MainAxisSize.max,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Flexible(
          flex: 1,
          child:
              (appState.device == mobile
                  ? Row(
                    children: [
                      BackButton(
                        onPressed: () {
                          homeState.setCurrentSession(-1);
                        },
                      ),
                      sessionTitle,
                    ],
                  )
                  : Align(alignment: Alignment.center, child: sessionTitle)),
        ),
        const Flexible(flex: 10, child: SessionRecord()),
        const Flexible(
          flex: 2,
          child: Align(
            alignment: Alignment.bottomCenter,
            child: SingleChildScrollView(
              child: TextField(
                decoration: InputDecoration(hintText: "Type here..."),
                maxLines: null,
              ),
            ),
          ),
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
            mainAxisAlignment:
                (isMe ? MainAxisAlignment.end : MainAxisAlignment.start),
            children: [(isMe ? message : avatar), (isMe ? avatar : message)],
          ),
        );
      },
      itemCount: records.length,
    );
  }
}
