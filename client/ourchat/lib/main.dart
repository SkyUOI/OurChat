import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'join.dart';
import 'connection.dart';
import 'home.dart';
import 'config.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'dart:core';
import 'dart:io';
import 'package:logger/logger.dart';

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
    0: join
    1: home
  */
  int device = desktop;
  bool isWeb = kIsWeb;
  OurchatConfig? config;
  Logger logger = Logger();

  void init() async {
    logger.i("init Ourchat");
    config = OurchatConfig();
    config!.loadConfig();
    connection = OurchatConnection(dealWithEvent);
    connection!
        .setAddress(config!.data!["server_address"], config!.data!["ws_port"]);

    if (!isWeb) {
      if (!await Directory("./cache").exists()) {
        await Directory("./cache").create();
      }
    }
    logger.d("IsWeb: $isWeb");
    notifyListeners();
    logger.i("init Ourchat done");
  }

  void dealWithEvent(var data) {
    var tmp = [];
    for (var pair in listenQueue) {
      tmp.add(pair);
    }
    for (var pair in tmp) {
      if (pair["code"] == data["code"]) {
        pair["func"](data);
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
      home: LayoutBuilder(builder: (context, constraints) {
        appState.device =
            (constraints.maxHeight < constraints.maxWidth) ? desktop : mobile;
        return page;
      }),
      theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue)),
    );
  }
}
