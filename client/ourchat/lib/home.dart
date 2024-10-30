import 'package:flutter/material.dart';

class Home extends StatelessWidget {
  const Home({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
        body: Padding(
      padding: EdgeInsets.all(8.0),
      child: Row(
        children: [
          Flexible(
            flex: 1,
            child: Padding(
              padding: EdgeInsets.all(8.0),
              child: SessionList(),
            ),
          ),
          Flexible(
            flex: 3,
            child: SessionWidget(),
          )
        ],
      ),
    ));
  }
}

class SessionList extends StatefulWidget {
  const SessionList({
    super.key,
  });

  @override
  State<SessionList> createState() => _SessionListState();
}

class _SessionListState extends State<SessionList> {
  List<Map<String, String>> sessions = [
    {"name": "username1", "image": "assets/images/logo.png"},
    {"name": "username2", "image": "assets/images/logo.png"},
    {"name": "username3", "image": "assets/images/logo.png"},
    {"name": "username4", "image": "assets/images/logo.png"},
    {"name": "username5", "image": "assets/images/logo.png"},
    {"name": "username6", "image": "assets/images/logo.png"},
    {"name": "username7", "image": "assets/images/logo.png"},
    {"name": "username8", "image": "assets/images/logo.png"},
    {"name": "username9", "image": "assets/images/logo.png"},
    {"name": "username10", "image": "assets/images/logo.png"}
  ];
  var currentIndex = -1;
  var hoverIndex = -1;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const TextField(
          decoration: InputDecoration(hintText: "Search"),
        ),
        Expanded(
            child: ListView.builder(
          itemBuilder: (context, index) {
            return MouseRegion(
              child: GestureDetector(
                child: Container(
                  padding: const EdgeInsets.all(10.0),
                  color: (index == currentIndex
                      ? Theme.of(context).focusColor
                      : (index == hoverIndex
                          ? Theme.of(context).secondaryHeaderColor
                          : Theme.of(context).canvasColor)),
                  child: Row(
                    children: [
                      Image.asset(
                        "assets/images/logo.png",
                        height: 50.0,
                      ),
                      Text(sessions[index]["name"].toString())
                    ],
                  ),
                ),
                onTap: () {
                  setState(() {
                    currentIndex = index;
                  });
                },
              ),
              onEnter: (e) {
                setState(() {
                  hoverIndex = index;
                });
              },
              onExit: (e) {
                setState(() {
                  hoverIndex = -1;
                });
              },
            );
          },
          itemCount: 10,
        ))
      ],
    );
  }
}

class SessionWidget extends StatelessWidget {
  const SessionWidget({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return const Column(
      mainAxisSize: MainAxisSize.max,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Flexible(
          flex: 1,
          child: Align(
            alignment: Alignment.topCenter,
            child: Text(
              "Session",
              style: TextStyle(fontSize: 30),
            ),
          ),
        ),
        Flexible(flex: 10, child: SessionRecord()),
        Flexible(
            flex: 2,
            child: Align(
              alignment: Alignment.bottomCenter,
              child: SingleChildScrollView(
                child: TextField(
                  decoration: InputDecoration(hintText: "Type here..."),
                  maxLines: null,
                ),
              ),
            ))
      ],
    );
  }
}

class SessionRecord extends StatefulWidget {
  const SessionRecord({
    super.key,
  });

  @override
  State<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends State<SessionRecord> {
  List<List> records = [
    [
      "User1",
      [const Text("Message1"), const Text("Message1_newLine")],
      true
    ], // username messages isMe
    [
      "User2",
      [const Text("Message2")],
      false
    ],
    [
      "User3",
      [const Text("Message3")],
      false
    ],
    [
      "User4",
      [const Text("Message4")],
      false
    ],
    [
      "User5",
      [const Text("Message5")],
      false
    ],
  ];
  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemBuilder: (context, index) {
        String username = records[index][0];
        List<Widget> messages = records[index][1];
        bool isMe = records[index][2];
        Widget avatar = Image.asset(
          "assets/images/logo.png",
          height: 30.0,
        );
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
                  )),
            )
          ],
        );
        return Container(
            margin: const EdgeInsets.all(5.0),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment:
                  (isMe ? MainAxisAlignment.end : MainAxisAlignment.start),
              children: [(isMe ? message : avatar), (isMe ? avatar : message)],
            ));
      },
      itemCount: records.length,
    );
  }
}
