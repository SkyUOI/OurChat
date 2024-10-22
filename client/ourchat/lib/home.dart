import 'package:flutter/material.dart';

class Home extends StatefulWidget {
  const Home({
    super.key,
  });

  @override
  State<Home> createState() => _HomeState();
}

class _HomeState extends State<Home> {
  var sessions = [
    {"name": "username1", "image": "assets/images/logo.png", "focus": false},
    {"name": "username2", "image": "assets/images/logo.png", "focus": false},
    {"name": "username3", "image": "assets/images/logo.png", "focus": true},
    {"name": "username4", "image": "assets/images/logo.png", "focus": false},
    {"name": "username5", "image": "assets/images/logo.png", "focus": false},
    {"name": "username6", "image": "assets/images/logo.png", "focus": false},
    {"name": "username7", "image": "assets/images/logo.png", "focus": false},
    {"name": "username8", "image": "assets/images/logo.png", "focus": false},
    {"name": "username9", "image": "assets/images/logo.png", "focus": false},
    {"name": "username10", "image": "assets/images/logo.png", "focus": false}
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Column(
      children: [
        const TextField(
          decoration: InputDecoration(labelText: "Search"),
        ),
        Expanded(
            child: ListView.builder(
          itemBuilder: (context, index) {
            return Container(
              padding: const EdgeInsets.all(10.0),
              color: (sessions[index]["focus"] == true
                  ? Theme.of(context).focusColor
                  : Theme.of(context).canvasColor),
              // color: Theme.of(context).secondaryHeaderColor,
              child: Row(
                children: [
                  Image.asset(
                    "assets/images/logo.png",
                    height: 50.0,
                  ),
                  Text(sessions[index]["name"].toString())
                ],
              ),
            );
          },
          itemCount: 10,
        ))
      ],
    ));
  }
}
