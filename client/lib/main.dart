import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/session.dart';
import 'core/account.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/event.dart';
import 'core/log.dart';
import 'dart:core';

void main() async {
  await initLocalStorage();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  int device = desktop;
  OurchatServer? server;
  OurchatAccount? thisAccount;
  late database.PublicOurchatDatabase publicDB;
  database.OurchatDatabase? privateDB;
  OurchatEventSystem? eventSystem;
  OurchatConfig config;
  Map<Int64, OurchatAccount> accountCachePool = {};
  Map<Int64, OurchatSession> sessionCachePool = {};

  OurchatAppState() : config = OurchatConfig() {
    logger.i("init Ourchat");
    constructLogger(convertStrIntoLevel(config["log_level"]));
    publicDB = database.PublicOurchatDatabase();
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
          appState.device = (constraints.maxHeight < constraints.maxWidth)
              ? desktop
              : mobile; // 通过屏幕比例判断桌面端/移动端
          return ServerSetting();
        },
      ),
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Color(appState.config["color"]),
        ),
      ),
    );
  }
}
