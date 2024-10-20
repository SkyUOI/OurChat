import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'join.dart';
import 'connection.dart';
import 'home.dart';
import 'config.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

void main() async {
  await initLocalStorage();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  OurchatConnection? connection;
  List listenQueue = [
    // {"code":0,"func":func}
  ];
  int where = joinUi;
  /*
    0: login
    1: home
  */
  OurchatConfig? config;

  void init() {
    config = OurchatConfig();
    config!.loadConfig();
    connection = OurchatConnection(dealWithMessage);
    connection!
        .setAddress(config!.data!["server_ip"], config!.data!["ws_port"]);
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
        var appState = OurchatAppState();
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
    var appState = context.watch<OurchatAppState>();
    Widget page;
    if (appState.where == joinUi) {
      page = const Join();
    } else if (appState.where == homeUi) {
      page = const Home();
    } else {
      page = const Placeholder();
    }

    return MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: page,
      theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue)),
    );
  }
}
