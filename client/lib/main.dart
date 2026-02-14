import 'dart:io';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/session.dart';
import 'package:ourchat/core/account.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/auth.dart';
import 'package:ourchat/home.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_single_instance/flutter_single_instance.dart';

// Conditionally import desktop-specific packages only when not on web
import 'package:window_manager/window_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/window_manager_stub.dart';
import 'package:tray_manager/tray_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/tray_manager_stub.dart';

// import 'package:ourchat/core/stubs/window_manager_stub.dart';
// import 'package:ourchat/core/stubs/tray_manager_stub.dart';

import 'dart:core';
import 'dart:async';

final GlobalKey<ScaffoldMessengerState> rootScaffoldMessengerKey =
    GlobalKey<ScaffoldMessengerState>();
Timer flashTrayTimer = Timer.periodic(Duration.zero, (_) {});
bool trayStatus = true, isFlashing = false;
// true means icon is normal, false means icon is empty
OurChatConfig config = OurChatConfig();

void changeTrayIcon() {
  if (!kIsWeb) {
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
}

void startFlashTray() {
  if (isFlashing || kIsWeb) {
    return;
  }
  flashTrayTimer =
      Timer.periodic(Duration(milliseconds: 500), (_) => changeTrayIcon());
  isFlashing = true;
}

void stopFlashTray() {
  if (!kIsWeb && !isFlashing) {
    flashTrayTimer.cancel();
    trayStatus = true;
    trayManager.setIcon(Platform.isWindows
        ? "assets/images/logo_without_text.ico"
        : "assets/images/logo_without_text.png");
    isFlashing = false;
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (!kIsWeb && (Platform.isWindows || Platform.isLinux || Platform.isMacOS)) {
    await windowManager.ensureInitialized();
    if (!await FlutterSingleInstance().isFirstInstance()) {
      await FlutterSingleInstance().focus();
      exit(0);
    }
    WindowOptions windowOptions = const WindowOptions(
      minimumSize: Size(900, 600),
      center: true,
      skipTaskbar: false,
      title: "OurChat",
    );
    windowManager.waitUntilReadyToShow(windowOptions, () async {
      await windowManager.show();
    });
    config.prefsWithCache = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions());
    config.reload();
    constructLogger(convertStrIntoLevel(config["log_level"]));
  }
  runApp(const MainApp());
}

class OurChatAppState extends ChangeNotifier {
  int screenMode = desktop;
  OurChatServer? server;
  OurChatAccount? thisAccount;
  late database.PublicOurChatDatabase publicDB;
  database.OurChatDatabase? privateDB;
  OurChatEventSystem? eventSystem;
  late OurChatConfig config;
  Map<Int64, OurChatAccount> accountCachePool = {};
  Map<Int64, OurChatSession> sessionCachePool = {};
  List<Int64> gettingInfoAccountList = [], gettingInfoSessionList = [];
  late AppLocalizations l10n;

  OurChatAppState() {
    logger.i("init OurChat");
    publicDB = database.PublicOurChatDatabase();
    notifyListeners();
    logger.i("init OurChat done");
  }

  void update() {
    notifyListeners();
  }
}

class MainApp extends StatefulWidget {
  const MainApp({
    super.key,
  });

  @override
  State<MainApp> createState() => _MainAppState();
}

class _MainAppState extends State<MainApp> with WindowListener, TrayListener {
  bool inited = false;

  @override
  void initState() {
    super.initState();
    if (!kIsWeb &&
        (Platform.isWindows || Platform.isLinux || Platform.isMacOS)) {
      windowManager.addListener(this);
      windowManager.setPreventClose(true);
      trayManager.addListener(this);
      trayManager.setIcon(Platform.isWindows
          ? "assets/images/logo_without_text.ico"
          : "assets/images/logo_without_text.png");
      trayManager.setToolTip("OurChat");
    }
  }

  @override
  void dispose() {
    if (!kIsWeb) {
      windowManager.removeListener(this);
      trayManager.removeListener(this);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
        create: (_) => OurChatAppState(),
        child: MaterialApp(
          title: "OurChat",
          home: Scaffold(
            body: LayoutBuilder(builder: (context, constraints) {
              if (!inited) {
                OurChatAppState ourChatAppState =
                    context.watch<OurChatAppState>();
                ourChatAppState.screenMode =
                    (constraints.maxHeight < constraints.maxWidth)
                        ? desktop
                        : mobile; // 通过屏幕比例判断桌面端/移动端
                ourChatAppState.config = config;
                ourChatAppState.l10n = AppLocalizations.of(context)!;

                if (!kIsWeb) {
                  trayManager.setContextMenu(Menu(items: [
                    MenuItem(key: "show", label: ourChatAppState.l10n.show("")),
                    MenuItem(key: "exit", label: ourChatAppState.l10n.exit)
                  ]));
                }

                inited = true;
                if (ourChatAppState.config["recent_account"] != null &&
                    ourChatAppState.config["recent_password"] != null) {
                  WidgetsBinding.instance.addPostFrameCallback((_) {
                    Navigator.push(context,
                        MaterialPageRoute(builder: (context) => AutoLogin()));
                  });
                } else {
                  WidgetsBinding.instance.addPostFrameCallback((_) {
                    Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => ServerSetting()));
                  });
                }
              }
              return Scaffold();
            }),
          ),
          scaffoldMessengerKey: rootScaffoldMessengerKey,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          localeResolutionCallback: (locale, supportedLocales) {
            Locale useLanguage = Locale("en");
            Locale? setLanguage = locale;
            logger.i("config language: ${config["language"]}");
            if (config["language"] != null) {
              setLanguage = Locale.fromSubtags(
                  languageCode: config["language"][0],
                  scriptCode: config["language"][1],
                  countryCode: config["language"][2]);
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
            config["language"] = [
              useLanguage.languageCode,
              useLanguage.scriptCode,
              useLanguage.countryCode
            ];
            return useLanguage;
          },
          theme: ThemeData(
            fontFamily: kIsWeb ? null : (Platform.isWindows ? "微软雅黑" : null),
            useMaterial3: true,
            colorScheme: ColorScheme.fromSeed(
              seedColor: Color(config["color"]),
            ),
          ),
          darkTheme: ThemeData(
            brightness: Brightness.dark,
          ),
          themeMode: ThemeMode.system,
        ));
  }

  // Desktop-only window and tray event handlers
  @override
  void onWindowClose() async {
    if (!kIsWeb) {
      windowManager.hide();
      super.onWindowClose();
    }
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
    stopFlashTray();
    super.onTrayIconMouseDown();
  }
}

class AutoLogin extends StatelessWidget {
  const AutoLogin({
    super.key,
  });

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
        Navigator.pushReplacement(
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
        Navigator.pushReplacement(
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
      Navigator.pushReplacement(context, MaterialPageRoute(
        builder: (context) {
          return Home();
        },
      ));
    }
  }

  @override
  Widget build(BuildContext context) {
    var l10n = context.watch<OurChatAppState>().l10n;
    autoLogin(context);
    return Scaffold(
      body: Center(
          child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [CircularProgressIndicator(), Text(l10n.autoLogin)],
      )),
    );
  }
}
