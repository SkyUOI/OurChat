import 'package:flutter/material.dart';
import 'ourchat/ourchat_account.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:logger/logger.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/ourchat/ourchat_server.dart';
import 'dart:core';

void main() async {
  await initLocalStorage();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  int device = desktop;
  OurchatConfig? config;
  Logger logger = Logger();
  OurChatServer? server;
  OurchatAccount? thisAccount;

  void init() async {
    logger.i("init Ourchat");
    config = OurchatConfig();
    config!.loadConfig();

    notifyListeners();
    logger.i("init Ourchat done");
  }

  void update() {
    notifyListeners();
  }
}

class MainApp extends StatelessWidget {
  const MainApp({super.key});

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
  const Controller({super.key});

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurchatAppState>();
    return MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: LayoutBuilder(
        builder: (context, constraints) {
          appState.device =
              (constraints.maxHeight < constraints.maxWidth) ? desktop : mobile;
          return const Navigator(pages: [MaterialPage(child: ServerSetting())]);
        },
      ),
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Color(appState.config!.data!["color"]),
        ),
      ),
    );
  }
}
