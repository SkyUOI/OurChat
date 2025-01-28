import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:logger/logger.dart';
import 'const.dart';
import 'home.dart';
import 'config.dart';
import 'auth.dart';
import 'server_setting.dart';
import 'ourchat/ourchat_server.dart';
import 'dart:core';
import 'dart:io';

void main() async {
  await initLocalStorage();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  int where = serverSettingUi;
  int device = desktop;
  bool isWeb = kIsWeb;
  OurchatConfig? config;
  Logger logger = Logger();
  OurChatServer? server;

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

  void update() {
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
    Widget window = const Placeholder();
    if (appState.where == serverSettingUi) {
      window = const ServerSetting();
    } else if (appState.where == authUi) {
      window = const Auth();
    } else if (appState.where == homeUi) {
      window = const Home();
    }
    return MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: LayoutBuilder(builder: (context, constraints) {
        appState.device =
            (constraints.maxHeight < constraints.maxWidth) ? desktop : mobile;
        return window;
      }),
      theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(
              seedColor: Color(appState.config!.data!["color"]))),
    );
  }
}
