import 'package:flutter/material.dart';
import 'ourchat/ourchat_account.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/ourchat/ourchat_server.dart';
import 'log.dart';
import 'dart:core';

void main() async {
  await initLocalStorage();
  initConfig();
  await constructLogger();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  int device = desktop;
  OurChatServer? server;
  OurchatAccount? thisAccount;

  OurchatAppState() {
    logger.i("init Ourchat");

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
          seedColor: Color(ourchatConfig["color"]),
        ),
      ),
    );
  }
}
