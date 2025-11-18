import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/session.dart';
import 'package:ourchat/core/account.dart';
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
import 'package:window_manager/window_manager.dart';
import 'package:tray_manager/tray_manager.dart';
import 'dart:core';
import 'dart:io';
import 'dart:async';

final GlobalKey<ScaffoldMessengerState> rootScaffoldMessengerKey =
    GlobalKey<ScaffoldMessengerState>();
Timer flashTrayTimer = Timer.periodic(Duration.zero, (_) {});
bool trayStatus = true, isFlashing = false;
// true means icon is normal, false means icon is empty

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();

  WindowOptions windowOptions = const WindowOptions(
    minimumSize: Size(900, 600),
    center: true,
    skipTaskbar: false,
    title: "OurChat",
  );
  windowManager.waitUntilReadyToShow(windowOptions, () async {
    await windowManager.show();
  });
  runApp(const MainApp());
}

class OurChatAppState extends ChangeNotifier {
  int screenMode = desktop;
  OurChatServer? server;
  OurChatAccount? thisAccount;
  late database.PublicOurChatDatabase publicDB;
  database.OurChatDatabase? privateDB;
  OurChatEventSystem? eventSystem;
  OurChatConfig config;
  Map<Int64, OurChatAccount> accountCachePool = {};
  Map<Int64, OurChatSession> sessionCachePool = {};
  List<Int64> gettingInfoAccountList = [], gettingInfoSessionList = [];
  late AppLocalizations l10n;

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

void changeTrayIcon() {
  if (trayStatus) {
    trayManager.setIcon(Platform.isWindows
        ? "assets/images/empty.ico"
        : "assets/images/empty.png");
  } else {
    trayManager.setIcon(Platform.isWindows
        ? "assets/images/logo_without_text.ico"
        : "assets/images/logo_without_text.png");
  }
  trayStatus = !trayStatus;
}

void startFlashTray() {
  if (isFlashing) {
    return;
  }
  flashTrayTimer =
      Timer.periodic(Duration(milliseconds: 500), (_) => changeTrayIcon());
  isFlashing = true;
}

void stopFlashTray() {
  if (isFlashing = false) {
    flashTrayTimer.cancel();
    trayStatus = true;
    trayManager.setIcon(Platform.isWindows
        ? "assets/images/logo_without_text.ico"
        : "assets/images/logo_without_text.png");
    isFlashing = false;
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

class Controller extends StatefulWidget {
  const Controller({super.key});

  @override
  State<Controller> createState() => _ControllerState();
}

class _ControllerState extends State<Controller>
    with WindowListener, TrayListener {
  bool logined = false;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
    windowManager.setPreventClose(true);
    trayManager.addListener(this);
    trayManager.setIcon(Platform.isWindows
        ? "assets/images/logo_without_text.ico"
        : "assets/images/logo_without_text.png");
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    trayManager.removeListener(this);
    super.dispose();
  }

  Future autoLogin(BuildContext context) async {
    logger.i("AUTO login");
    var ourchatAppState = context.watch<OurChatAppState>();
    bool isTLS = await OurChatServer.tlsEnabled(
        ourchatAppState.config["servers"][0]["host"],
        ourchatAppState.config["servers"][0]["port"]);
    OurChatServer server = OurChatServer(
        ourchatAppState.config["servers"][0]["host"],
        ourchatAppState.config["servers"][0]["port"],
        isTLS);
    var connectRes = await server.getServerInfo();
    if (connectRes != okStatusCode) {
      logger.w("fiailed to connect to server");
      if (context.mounted) {
        Navigator.push(
            context, MaterialPageRoute(builder: (context) => ServerSetting()));
      }
      return;
    }
    ourchatAppState.server = server;
    OurChatAccount ocAccount = OurChatAccount(ourchatAppState);
    String? email, ocid;
    if (ourchatAppState.config["recent_account"].contains('@')) {
      // 判断邮箱/ocid登录
      email = ourchatAppState.config["recent_account"];
    } else {
      ocid = ourchatAppState.config["recent_account"];
    }

    bool loginRes = await ocAccount.login(
      ourchatAppState.config["recent_password"],
      ocid,
      email,
    );
    if (!loginRes) {
      logger.w("failed to auto-login");
      if (context.mounted) {
        Navigator.push(
            context, MaterialPageRoute(builder: (context) => Auth()));
      }
      return;
    }

    ourchatAppState.thisAccount = ocAccount;
    ourchatAppState.privateDB = database.OurChatDatabase(ocAccount.id);
    ourchatAppState.eventSystem = OurChatEventSystem(ourchatAppState);
    await ourchatAppState.thisAccount!.getAccountInfo();

    ourchatAppState.eventSystem!.listenEvents();
    ourchatAppState.update();
    if (context.mounted) {
      // 跳转主界面
      Navigator.pop(context);
      Navigator.push(context, MaterialPageRoute(
        builder: (context) {
          return Home();
        },
      ));
    }
  }

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    return MaterialApp(
      scaffoldMessengerKey: rootScaffoldMessengerKey,
      localeResolutionCallback: (locale, supportedLocales) {
        Locale useLanguage = Locale("en");
        Locale? setLanguage = locale;
        if (ourchatAppState.config["language"] != null) {
          setLanguage = Locale.fromSubtags(
              languageCode: ourchatAppState.config["language"][0],
              scriptCode: ourchatAppState.config["language"][1],
              countryCode: ourchatAppState.config["language"][2]);
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
        ourchatAppState.config["language"] = [
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
          var l10n = ourchatAppState.l10n = AppLocalizations.of(context)!;
          trayManager.setContextMenu(Menu(items: [
            MenuItem(key: "show", label: l10n.show("")),
            MenuItem(key: "exit", label: l10n.exit)
          ]));
          ourchatAppState.screenMode =
              (constraints.maxHeight < constraints.maxWidth)
                  ? desktop
                  : mobile; // 通过屏幕比例判断桌面端/移动端
          if (ourchatAppState.config["recent_password"].isNotEmpty) {
            if (!logined) {
              autoLogin(context);
              logined = true;
            }
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
            return ServerSetting();
          }
        },
      ),
      theme: ThemeData(
        fontFamily: Platform.isWindows ? "微软雅黑" : null,
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Color(ourchatAppState.config["color"]),
        ),
      ),
    );
  }

  @override
  void onWindowClose() async {
    windowManager.hide();
    super.onWindowClose();
  }

  @override
  void onTrayIconRightMouseDown() {
    trayManager.popUpContextMenu();
    super.onTrayIconRightMouseDown();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    if (menuItem.key == "show") {
      windowManager.show();
      windowManager.focus();
      stopFlashTray();
    } else if (menuItem.key == "exit") {
      trayManager.destroy();
      windowManager.destroy();
    }
    super.onTrayMenuItemClick(menuItem);
  }

  @override
  void onTrayIconMouseDown() {
    windowManager.show();
    windowManager.focus();
    stopFlashTray();
    super.onTrayIconMouseDown();
  }
}
