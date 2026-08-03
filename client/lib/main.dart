import 'dart:convert';
import 'dart:io';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/auth_notifier.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/secret_store.dart';
import 'package:ourchat/home.dart';
import 'package:shared_preferences/shared_preferences.dart';
// Conditionally import desktop-specific packages only when not on web
import 'package:flutter_single_instance/flutter_single_instance.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/flutter_single_instance.dart';
import 'package:window_manager/window_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/window_manager_stub.dart';
import 'package:tray_manager/tray_manager.dart'
    if (dart.library.html) 'package:ourchat/core/stubs/tray_manager_stub.dart';

import 'dart:core';
import 'dart:async';

part 'main.g.dart';

final GlobalKey<ScaffoldMessengerState> rootScaffoldMessengerKey =
    GlobalKey<ScaffoldMessengerState>();
Timer flashTrayTimer = Timer.periodic(Duration.zero, (_) {});
bool trayStatus = true, isFlashing = false;
// true means icon is normal, false means icon is empty

void changeTrayIcon() {
  if (!kIsWeb) {
    if (trayStatus) {
      trayManager
          .setIcon(
            Platform.isWindows
                ? "assets/images/empty.ico"
                : "assets/images/empty.png",
          )
          .catchError((_) {});
    } else {
      trayManager
          .setIcon(
            Platform.isWindows
                ? "assets/images/logo_without_text.ico"
                : "assets/images/logo_without_text.png",
          )
          .catchError((_) {});
    }
    trayStatus = !trayStatus;
  }
}

void startFlashTray() {
  if (isFlashing || kIsWeb) {
    return;
  }
  flashTrayTimer = Timer.periodic(
    Duration(milliseconds: 500),
    (_) => changeTrayIcon(),
  );
  isFlashing = true;
}

void stopFlashTray() {
  if (!kIsWeb && isFlashing) {
    flashTrayTimer.cancel();
    trayStatus = true;
    trayManager
        .setIcon(
          Platform.isWindows
              ? "assets/images/logo_without_text.ico"
              : "assets/images/logo_without_text.png",
        )
        .catchError((_) {});
    isFlashing = false;
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  initDB();
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
  }
  final prefs = await SharedPreferencesWithCache.create(
    cacheOptions: const SharedPreferencesWithCacheOptions(),
  );
  var config = OurChatConfig.defaults.copyWith(prefsWithCache: prefs);
  // load from prefs if available
  final stored = prefs.getString('config');
  if (stored != null) {
    final loaded = OurChatConfig.fromJson(jsonDecode(stored));
    config = loaded.copyWith(
      prefsWithCache: prefs,
      // ensure at least one server exists
      servers: loaded.servers.isNotEmpty
          ? loaded.servers
          : OurChatConfig.defaults.servers,
    );
  } else {
    config.saveConfig(); // persist defaults on first launch
  }
  constructLogger(convertStrIntoLevel(config.logLevel));
  runApp(const ProviderScope(child: MainApp()));
}

void initDB() {
  var db = database.PublicOurChatDatabase();
  publicDB = db;
}

late AppLocalizations l10n;

late database.PublicOurChatDatabase publicDB;
database.OurChatDatabase? privateDB;

@riverpod
class ScreenModeNotifier extends _$ScreenModeNotifier {
  @override
  ScreenMode build() {
    return ScreenMode.desktop;
  }

  void switchMode(ScreenMode mode) {
    if (mode != state) {
      state = mode;
    }
  }
}

@Riverpod(keepAlive: true)
class ThisAccountIdNotifier extends _$ThisAccountIdNotifier {
  @override
  Int64? build() {
    return null;
  }

  void setAccountId(Int64? id) {
    state = id;
  }

  void clear() {
    state = null;
  }
}

@Riverpod(keepAlive: true)
class OurChatServerNotifier extends _$OurChatServerNotifier {
  @override
  OurChatServer build() {
    // After a login the "current" server is the active instance's connection.
    final key = ref.watch(activeAccountProvider);
    if (key != null) {
      final inst = ref.watch(instancesProvider)[key];
      if (inst != null) return inst.server;
    }
    final config = ref.read(configProvider);
    final server = config.servers.isNotEmpty
        ? config.servers[0]
        : ServerConfig(host: 'skyuoi.org', port: 7777);
    return OurChatServer(server.host, server.port, false);
  }

  void update(OurChatServer server) {
    state = server;
  }
}

class MainApp extends ConsumerStatefulWidget {
  const MainApp({super.key});

  @override
  ConsumerState<MainApp> createState() => _MainAppState();
}

class _MainAppState extends ConsumerState<MainApp>
    with WindowListener, TrayListener {
  bool inited = false;

  @override
  void initState() {
    super.initState();
    if (!kIsWeb &&
        (Platform.isWindows || Platform.isLinux || Platform.isMacOS)) {
      windowManager.addListener(this);
      windowManager.setPreventClose(true);
      trayManager.addListener(this);
      trayManager
          .setIcon(
            Platform.isWindows
                ? "assets/images/logo_without_text.ico"
                : "assets/images/logo_without_text.png",
          )
          .catchError((_) {});
      trayManager.setToolTip("OurChat").catchError((_) {});
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
    final config = ref.read(configProvider);
    return MaterialApp(
      title: "OurChat",
      home: Scaffold(
        body: LayoutBuilder(
          builder: (context, constraints) {
            l10n = AppLocalizations.of(context)!;
            ref
                .read(screenModeProvider.notifier)
                .switchMode(
                  (constraints.maxHeight < constraints.maxWidth)
                      ? ScreenMode.desktop
                      : ScreenMode.mobile,
                ); // Determine desktop/mobile by aspect ratio
            if (!inited) {
              if (!kIsWeb) {
                trayManager
                    .setContextMenu(
                      Menu(
                        items: [
                          MenuItem(key: "show", label: l10n.show("")),
                          MenuItem(key: "exit", label: l10n.exit),
                        ],
                      ),
                    )
                    .catchError((_) {});
              }

              inited = true;
              if (config.savedAccounts.any((a) => a.autoLogin)) {
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => AutoLogin()),
                  );
                });
              } else {
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => ServerSetting()),
                  );
                });
              }
            }
            return Home();
          },
        ),
      ),
      scaffoldMessengerKey: rootScaffoldMessengerKey,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      localeResolutionCallback: (locale, supportedLocales) {
        Locale useLanguage = Locale("en");
        Locale? setLanguage = locale;
        logger.i("config language: ${config.language}");
        if (config.language != null &&
            config.language!.languageCode.isNotEmpty) {
          final lang = config.language!;
          setLanguage = Locale.fromSubtags(
            languageCode: lang.languageCode,
            scriptCode: lang.scriptCode.isNotEmpty ? lang.scriptCode : null,
            countryCode: lang.countryCode.isNotEmpty ? lang.countryCode : null,
          );
        }
        for (int i = 0; i < supportedLocales.length; i++) {
          var availableLanguage = supportedLocales.elementAt(i);
          if (availableLanguage.languageCode == setLanguage!.languageCode) {
            useLanguage = availableLanguage;
            break;
          }
        }
        logger.i(
          "use language (${useLanguage.languageCode},${useLanguage.scriptCode},${useLanguage.countryCode})",
        );
        final newLang = LanguageConfig(
          languageCode: useLanguage.languageCode,
          scriptCode: useLanguage.scriptCode ?? '',
          countryCode: useLanguage.countryCode ?? '',
        );
        if (config.language != newLang) {
          Future(() => ref.read(configProvider.notifier).setLanguage(newLang));
        }
        return useLanguage;
      },
      theme: ThemeData(
        fontFamily: kIsWeb ? null : (Platform.isWindows ? "微软雅黑" : null),
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(seedColor: Color(config.color)),
      ),
      darkTheme: ThemeData(brightness: Brightness.dark),
      themeMode: ThemeMode.system,
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
    trayManager.popUpContextMenu().catchError((_) {});
    super.onTrayIconRightMouseDown();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    if (menuItem.key == "show") {
      windowManager.show();
      stopFlashTray();
    } else if (menuItem.key == "exit") {
      trayManager.destroy().catchError((_) {});
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

class AutoLogin extends ConsumerStatefulWidget {
  const AutoLogin({super.key});

  @override
  ConsumerState<AutoLogin> createState() => _AutoLoginState();
}

/// Switch the UI's focused account to [key]: update the active-account
/// pointer, the legacy globals (privateDB / thisAccountId) that existing call
/// sites read, and the persisted config. No-op when [key] has no live
/// instance.
void switchActive(WidgetRef ref, AccountKey key) {
  final inst = ref.read(instancesProvider)[key];
  if (inst == null) return;
  ref.read(activeAccountProvider.notifier).set(key);
  privateDB = inst.privateDB;
  ref.read(thisAccountIdProvider.notifier).setAccountId(inst.accountId);
  ref
      .read(configProvider.notifier)
      .setActiveAccount(key.serverId, key.accountId.toInt());
}

class _AutoLoginState extends ConsumerState<AutoLogin> {
  bool triedAutoLogin = false;

  /// Login one saved account, building its runtime instance + event system.
  /// Returns true on success. Serialized by the caller to avoid racing the
  /// shared auth state.
  Future<bool> _loginOne(SavedAccount acc, OurChatConfig cfg) async {
    final sc = cfg.servers.firstWhere(
      (s) => s.uniqueIdentifier == acc.serverId,
      orElse: () => ServerConfig(host: 'skyuoi.org', port: 7777),
    );
    final server = OurChatServer(sc.host, sc.port, sc.isTLS ?? false);
    final connectRes = await server.getServerInfo();
    if (connectRes != okStatusCode) {
      logger.w("auto-login: failed to connect to ${sc.host}:${sc.port}");
      return false;
    }
    final serverId = server.uniqueIdentifier!;
    final accountIdent = acc.email ?? acc.ocid;
    if (accountIdent == null) return false;
    final password = await SecretStore.readCredential(serverId, accountIdent);
    if (password == null || password.isEmpty) return false;

    String? email, ocid;
    if (accountIdent.contains('@')) {
      email = accountIdent;
    } else {
      ocid = accountIdent;
    }

    final ok = await ref
        .read(authProvider.notifier)
        .login(password: password, email: email, ocid: ocid, server: server);
    if (!ok) return false;
    final accountId = ref.read(authProvider).accountId!;
    logger.i("auto-login successful, account ID: $accountId");

    final newPrivateDB = database.OurChatDatabase(serverId, accountId);
    final instance = OurChatInstance(
      serverId: serverId,
      accountId: accountId,
      server: server,
      privateDB: newPrivateDB,
    );
    ref.read(instancesProvider.notifier).add(instance);

    ref
        .read(configProvider.notifier)
        .upsertSavedAccount(
          SavedAccount(
            serverId: serverId,
            accountId: accountId.toInt(),
            ocid: acc.ocid ?? ref.read(authProvider).ocid,
            email: acc.email ?? email,
            avatarKey: acc.avatarKey,
            lastLoginAt: DateTime.now(),
            autoLogin: true,
          ),
        );

    ref
        .read(ourChatEventSystemProvider(serverId, accountId).notifier)
        .listenEvents();
    ref
        .read(ourChatAccountProvider(serverId, accountId).notifier)
        .getAccountInfo();
    return true;
  }

  Future<void> autoLogin(BuildContext context) async {
    logger.i("AUTO login (multi)");
    final cfg = ref.read(configProvider);
    final autoAccounts = cfg.savedAccounts.where((a) => a.autoLogin).toList();
    if (autoAccounts.isEmpty) {
      logger.i("no saved account, redirect to server setting");
      if (context.mounted) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (context) => ServerSetting()),
        );
      }
      return;
    }

    // Serial login (avoids racing the shared auth state), building one live
    // instance per account.
    final successes = <SavedAccount>[];
    for (final acc in autoAccounts) {
      if (await _loginOne(acc, cfg)) successes.add(acc);
    }

    if (successes.isEmpty) {
      logger.w("auto-login: no account succeeded, redirect to server setting");
      if (context.mounted) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (context) => ServerSetting()),
        );
      }
      return;
    }

    // Choose the active account: the config-preferred one if it succeeded,
    // otherwise the first success.
    final active = successes.firstWhere(
      (a) =>
          a.serverId == cfg.activeServerId &&
          a.accountId == cfg.activeAccountId,
      orElse: () => successes.first,
    );
    switchActive(ref, AccountKey(active.serverId, Int64(active.accountId)));

    if (context.mounted) {
      Navigator.pushReplacement(
        context,
        MaterialPageRoute(builder: (context) => Home()),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    if (!triedAutoLogin) {
      autoLogin(context);
      triedAutoLogin = true;
    }
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [CircularProgressIndicator(), Text(l10n.autoLogin)],
        ),
      ),
    );
  }
}
