import 'package:flutter/material.dart';
import 'package:ourchat/main.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'dart:async';

// Conditionally import desktop-specific packages only when not on web
import 'package:window_manager/window_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/window_manager_stub.dart';
import 'package:tray_manager/tray_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/tray_manager_stub.dart';

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

class Launch extends StatefulWidget {
  const Launch({
    super.key,
  });

  @override
  State<Launch> createState() => _LaunchState();
}

class _LaunchState extends State<Launch> with WindowListener, TrayListener {
  Future initConfig(OurChatAppState appState) async {
    appState.config.prefsWithCache = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions());
  }

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
      home: Scaffold(
        body: FutureBuilder(
            future: initConfig(ourchatAppState),
            builder: (context, snapshot) {
              if (!inited && snapshot.connectionState == ConnectionState.done) {
                inited = true;
                ourchatAppState.config.reload();
                constructLogger(
                    convertStrIntoLevel(ourchatAppState.config["log_level"]));
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  Navigator.push(context,
                      MaterialPageRoute(builder: (context) => Controller()));
                });
              }
              return Center(
                child: CircularProgressIndicator(),
              );
            }),
      ),
      theme: ThemeData(
        fontFamily: kIsWeb ? null : (Platform.isWindows ? "微软雅黑" : null),
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Color(ourchatAppState.config["color"]),
        ),
      ),
    );
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
