import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:provider/provider.dart';
import 'join.dart';
import 'connection.dart';
import 'home.dart';

void main() {
  runApp(const MainApp());
}

class OurChatAppState extends ChangeNotifier {
  Connection? connection;
  var listenQueue = [
    // {"code":0,"func":func}
  ];
  var where = joinUi;
  /*
    0: login
    1: home
  */

  void init() {
    connection = Connection(dealWithMessage);
  }

  void dealWithMessage(var messageData) {
    var tmp = [];
    for (var pair in listenQueue) {
      tmp.add(pair);
    }
    for (var pair in tmp) {
      if (pair["code"] == messageData["code"]) {
        pair["func"](messageData);
      }
    }
  }

  void listen(var code, var func) {
    listenQueue.add({"code": code, "func": func});
  }

  void unlisten(var code, var func) {
    listenQueue
        .removeWhere((value) => value["code"] == code && value["func"] == func);
  }

  void toSomewhere(var id) {
    where = id;
    notifyListeners();
  }
}

class MainApp extends StatelessWidget {
  const MainApp({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) {
        var appState = OurChatAppState();
        appState.init();
        return appState;
      },
      child: const Controller(),
    );
  }
}

class Controller extends StatelessWidget {
  const Controller({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurChatAppState>();
    Widget page;
    if (appState.where == joinUi) {
      page = const Join();
    } else if (appState.where == homeUi) {
      page = const Home();
    } else {
      page = const Placeholder();
    }

    return MaterialApp(
      home: page,
      theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue)),
    );
  }
}
