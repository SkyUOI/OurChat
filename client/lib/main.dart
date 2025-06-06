import 'package:flutter/material.dart';
import 'package:ourchat/ourchat/ourchat_chore.dart';
import 'package:ourchat/ourchat/ourchat_database.dart' as database;
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'ourchat/ourchat_account.dart';
import 'package:provider/provider.dart';
import 'package:localstorage/localstorage.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/ourchat/ourchat_server.dart';
import 'log.dart';
import 'dart:core';

void main() async {
  await initLocalStorage();
  runApp(const MainApp());
}

class OurchatAppState extends ChangeNotifier {
  int device = desktop;
  OurChatServer? server;
  OurchatAccount? thisAccount;
  late database.PublicOurchatDatabase publicDB;
  database.OurchatDatabase? privateDB;
  OurchatConfig config;

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

  void listenMsgs() async {
    var stub = OurChatServiceClient(server!.channel!,
        interceptors: [server!.interceptor!]);
    var res = stub.fetchMsgs(
        FetchMsgsRequest(time: thisAccount!.latestMsgTime.timestamp));
    res.listen((res) {
      thisAccount!.latestMsgTime = OurchatTime(inputTimestamp: res.time);
      thisAccount!.updateLatestMsgTime();
      // TODO: 管理消息
    });
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
          return const Navigator(pages: [MaterialPage(child: ServerSetting())]);
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
