import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'welcome.dart';
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
  int where = welcomeUi;
  /*
    0: welcome
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

    if (!isWeb) {
      if (!await Directory("./cache").exists()) {
        await Directory("./cache").create();
      }
    }
    logger.d("IsWeb: $isWeb");
    notifyListeners();
    logger.i("init Ourchat done");
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
    if (appState.where == welcomeUi) {
      page = const Welcome();
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
