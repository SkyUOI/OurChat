import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/session.dart';
import 'core/account.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/launch.dart';
import 'package:ourchat/auth.dart';
import 'package:ourchat/home.dart';
import 'dart:core';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const MainApp());
}

class OurChatAppState extends ChangeNotifier {
  int device = desktop;
  OurChatServer? server;
  OurChatAccount? thisAccount;
  late database.PublicOurChatDatabase publicDB;
  database.OurChatDatabase? privateDB;
  OurChatEventSystem? eventSystem;
  OurChatConfig config;
  Map<Int64, OurChatAccount> accountCachePool = {};
  Map<Int64, OurChatSession> sessionCachePool = {};

  OurChatAppState() : config = OurChatConfig() {
    logger.i("init OurChat");
    publicDB = database.PublicOurChatDatabase();
    notifyListeners();
    logger.i("init OurChat done");
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
        var appState = OurChatAppState();
        return appState;
      },
      child: Launch(),
    );
  }
}

class Controller extends StatelessWidget {
  const Controller({super.key});

  Future autoLogin(BuildContext context) async {
    logger.i("AUTO login");
    var appState = context.watch<OurChatAppState>();
    bool isTLS = await OurChatServer.tlsEnabled(
        appState.config["servers"][0]["host"],
        appState.config["servers"][0]["port"]);
    OurChatServer server = OurChatServer(appState.config["servers"][0]["host"],
        appState.config["servers"][0]["port"], isTLS);
    var connectRes = await server.getServerInfo();
    if (connectRes != okStatusCode) {
      logger.w("fiailed to connect to server");
      if (context.mounted) {
        Navigator.push(
            context, MaterialPageRoute(builder: (context) => ServerSetting()));
      }
      return;
    }
    appState.server = server;
    OurChatAccount ocAccount = OurChatAccount(appState);
    String? email, ocid;
    if (appState.config["recent_account"].contains('@')) {
      // 判断邮箱/ocid登录
      email = appState.config["recent_account"];
    } else {
      ocid = appState.config["recent_account"];
    }
    var loginRes =
        await ocAccount.login(appState.config["recent_password"], ocid, email);
    if (loginRes.$1 != okStatusCode) {
      logger.w("failed to login");
      if (context.mounted) {
        Navigator.push(
            context, MaterialPageRoute(builder: (context) => Auth()));
      }
      return;
    }
    appState.thisAccount = ocAccount;
    appState.privateDB = database.OurChatDatabase(ocAccount.id);
    appState.eventSystem = OurChatEventSystem(appState);
    await appState.thisAccount!.getAccountInfo();
    appState.eventSystem!.listenEvents();
    appState.update();
    if (context.mounted) {
      // 跳转主界面
      Navigator.pop(context);
      Navigator.push(context, MaterialPageRoute(
        builder: (context) {
          return const Scaffold(
            body: Home(),
          );
        },
      ));
    }
  }

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurChatAppState>();
    return MaterialApp(
      localeResolutionCallback: (locale, supportedLocales) {
        Locale useLanguage = Locale("en");
        Locale? setLanguage = locale;
        if (appState.config["language"] != null) {
          setLanguage = Locale.fromSubtags(
              languageCode: appState.config["language"][0],
              scriptCode: appState.config["language"][1],
              countryCode: appState.config["language"][2]);
        }
        for (int i = 0; i < supportedLocales.length; i++) {
          var availableLanguage = supportedLocales.elementAt(i);
          if (availableLanguage.languageCode == setLanguage!.languageCode) {
            useLanguage = availableLanguage;
            break;
          }
        }
        logger.i(
            "use language (${useLanguage.languageCode},${useLanguage.scriptCode},${useLanguage.countryCode})");
        appState.config["language"] = [
          useLanguage.languageCode,
          useLanguage.scriptCode,
          useLanguage.countryCode
        ];
        return useLanguage;
      },
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: LayoutBuilder(
        builder: (context, constraints) {
          var l10n = AppLocalizations.of(context)!;
          appState.device = (constraints.maxHeight < constraints.maxWidth)
              ? desktop
              : mobile; // 通过屏幕比例判断桌面端/移动端
          if (appState.config["recent_password"].isNotEmpty) {
            logger.i("AUTO login");
            autoLogin(context);
            return Scaffold(
              body: Center(
                  child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  CircularProgressIndicator(),
                  Text(l10n.autoLogin,
                      style: TextStyle(
                          fontSize: AppStyles.smallFontSize,
                          color: Theme.of(context).hintColor))
                ],
              )),
            );
          } else {
            return SafeArea(child: ServerSetting());
          }
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
