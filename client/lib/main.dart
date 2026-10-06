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
import 'package:ourchat/auth.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/version_check.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/notification_service.dart';
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
final GlobalKey<NavigatorState> rootNavigatorKey = GlobalKey<NavigatorState>();

/// Announcement event ids for which a dialog has already been shown, so the
/// same announcement never pops up twice.
final Set<int> shownAnnouncementEventIds = {};

/// Show an [AlertDialog] for a freshly received [AnnouncementResponseEvent]
/// when the app is in the foreground. A no-op when the same announcement was
/// already shown, when the app is backgrounded, or before the navigator is
/// mounted.
void maybeShowAnnouncementDialog(AnnouncementResponseEvent event) {
  final eventId = event.eventId?.toInt();
  if (eventId == null) return;
  if (!shownAnnouncementEventIds.add(eventId)) {
    // Already shown once — never repeat the same announcement.
    return;
  }
  final lifecycleState = WidgetsBinding.instance.lifecycleState;
  if (lifecycleState == AppLifecycleState.paused ||
      lifecycleState == AppLifecycleState.detached) {
    return;
  }
  final context = rootNavigatorKey.currentContext;
  if (context == null) return;
  showDialog(
    context: context,
    builder: (context) {
      final time = event.sendTime?.datetime.toLocal();
      return AlertDialog(
        title: Text(
          (event.title?.isNotEmpty ?? false) ? event.title! : l10n.announcement,
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (event.content?.isNotEmpty ?? false)
              SelectableText(event.content!),
            const SizedBox(height: 8),
            Text(
              l10n.announcementPublisherId(event.publisherId.toString()),
              style: const TextStyle(color: Colors.grey, fontSize: 12),
            ),
            if (time != null)
              Text(
                '${time.year}-${time.month.toString().padLeft(2, '0')}-${time.day.toString().padLeft(2, '0')} '
                '${time.hour.toString().padLeft(2, '0')}:${time.minute.toString().padLeft(2, '0')}',
                style: const TextStyle(color: Colors.grey, fontSize: 12),
              ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(l10n.close),
          ),
        ],
      );
    },
  );
}

/// Tray-flash timer. Nullable on purpose: the previous top-level initializer
/// `Timer.periodic(Duration.zero, ...)` spun ~millions of ticks per second
/// from library load (app start / tests) until replaced — see issue #199.
Timer? flashTrayTimer;
bool trayStatus = true, isFlashing = false;
// true means icon is normal, false means icon is empty

void changeTrayIcon() {
  if (isDesktopPlatform) {
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

/// Start flashing the tray icon to signal unread messages (desktop only).
/// Guarded by [isDesktopPlatform] — on mobile there is no tray, and the old
/// web-only guard would have left a periodic timer calling missing platform
/// channels there.
void startFlashTray() {
  if (isFlashing || !isDesktopPlatform) {
    return;
  }
  flashTrayTimer?.cancel();
  flashTrayTimer = Timer.periodic(
    Duration(milliseconds: 500),
    (_) => changeTrayIcon(),
  );
  isFlashing = true;
}

void stopFlashTray() {
  if (isDesktopPlatform && isFlashing) {
    flashTrayTimer?.cancel();
    flashTrayTimer = null;
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

/// True when running on a desktop OS (Windows/Linux/macOS) and not on the
/// web. Gates window/tray integration which only exists on desktop.
bool get isDesktopPlatform =>
    !kIsWeb &&
    (Platform.isWindows || Platform.isLinux || Platform.isMacOS);

/// The most recent [AppLifecycleState] reported to [_MainAppState], or null
/// before the first lifecycle event (issue #199). Prefer
/// [currentAppLifecycleState], which falls back to the binding's state.
AppLifecycleState? lastAppLifecycleState;

/// Current app lifecycle state: the last state observed by [_MainAppState],
/// falling back to what the engine binding reports. Null when no lifecycle
/// event has been seen yet — treated as "foreground" by callers (the app is
/// visibly running when it has just started).
AppLifecycleState? get currentAppLifecycleState =>
    lastAppLifecycleState ?? WidgetsBinding.instance.lifecycleState;

/// Whether the conversation UI is potentially visible right now (used by the
/// new-message notification logic, issue #199). Null lifecycle state counts
/// as foreground (see [currentAppLifecycleState]).
bool get appInForeground =>
    currentAppLifecycleState == null ||
    currentAppLifecycleState == AppLifecycleState.resumed;

/// What the desktop window close (X) button should do, derived from the
/// user's configured [CloseBehavior] (issue #203). Pure function so the
/// decision can be unit tested without a window.
CloseAction resolveWindowCloseAction(CloseBehavior closeBehavior) {
  return closeBehavior == CloseBehavior.exit
      ? CloseAction.exitApp
      : CloseAction.minimizeToTray;
}

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
    with WindowListener, TrayListener, WidgetsBindingObserver {
  bool inited = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    if (isDesktopPlatform) {
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
    // Initialize system notifications (no-op stub on web). Tapping a
    // notification brings the window back and stops the tray flash (issue
    // #199). TODO(#199): deep-link to the conversation from the payload.
    final notifications = ref.read(ourChatNotificationServiceProvider);
    notifications.onNotificationTap = (_) {
      if (isDesktopPlatform) {
        windowManager.show().catchError((_) {});
        stopFlashTray();
      }
    };
    notifications.init();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    if (!kIsWeb) {
      windowManager.removeListener(this);
      trayManager.removeListener(this);
    }
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    lastAppLifecycleState = state;
    super.didChangeAppLifecycleState(state);
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
                WidgetsBinding.instance.addPostFrameCallback((_) async {
                  await connectToOfficialServer(ref);
                  if (context.mounted) {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => Auth()),
                    );
                  }
                });
              }
            }
            return Home();
          },
        ),
      ),
      scaffoldMessengerKey: rootScaffoldMessengerKey,
      navigatorKey: rootNavigatorKey,
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
      // The close (X) button is configurable: minimize to tray (default) or
      // exit the app (issue #203).
      switch (resolveWindowCloseAction(ref.read(configProvider).closeBehavior)) {
        case CloseAction.minimizeToTray:
          await windowManager.hide();
        case CloseAction.exitApp:
          await _exitApp();
      }
      super.onWindowClose();
    }
  }

  /// Desktop quit sequence: tear down the tray icon then destroy the window.
  /// Shared by the tray "exit" item and the close button when configured to
  /// exit (issue #203).
  Future<void> _exitApp() async {
    stopFlashTray();
    await ref.read(ourChatNotificationServiceProvider).cancelAll();
    trayManager.destroy().catchError((_) {});
    windowManager.destroy();
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
      _exitApp();
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
  // Attention signals belong to the previously focused account: clear them so
  // stale notifications/flashes do not leak across the switch (issue #199).
  unawaited(ref.read(ourChatNotificationServiceProvider).cancelAll());
  stopFlashTray();
}

/// Connect to the default/official server (the first configured server, else
/// the built-in `skyuoi.org:7777`) and ensure it is identified, so the login
/// screen can authenticate against it. Returns true when the server is ready.
Future<bool> connectToOfficialServer(WidgetRef ref) async {
  final config = ref.read(configProvider);
  final sc = config.servers.isNotEmpty
      ? config.servers.first
      : ServerConfig(host: 'skyuoi.org', port: 7777);
  // Assume non-TLS unless the stored config says otherwise. Probing TLS here
  // (via tlsEnabled) breaks on web: the browser CORS-preflights an HTTPS
  // request to a plain-HTTP server, which replies without CORS headers.
  final isTLS = sc.isTLS ?? false;
  final server = OurChatServer(sc.host, sc.port, isTLS);
  ref.read(ourChatServerProvider.notifier).update(server);
  final res = await server.getServerInfo();
  if (res != okStatusCode) return false;
  warnOnIncompatibleServer(server);
  ref
      .read(configProvider.notifier)
      .upsertServer(
        ServerConfig(
          host: sc.host,
          port: sc.port,
          label: sc.label,
          uniqueIdentifier: server.uniqueIdentifier,
          isTLS: isTLS,
        ),
      );
  return true;
}

/// Version negotiation warnings (issue #16), advisory only. Shown at most
/// once per server per reason per app run so reconnect loops cannot spam.
final Set<String> _versionWarnedServers = {};
void warnOnIncompatibleServer(OurChatServer server) {
  if (kIsWeb) return; // no dialog root on the web bootstrap path
  final result = checkServerCompatibility(
    serverVersion: server.serverVersion,
    minimumClientVersion: server.minimumClientVersion,
    clientVersion: currentVersion,
  );
  if (result == ServerCompatibility.ok) return;
  final key =
      '${server.uniqueIdentifier ?? server.host}:${server.port}:$result';
  if (!_versionWarnedServers.add(key)) return;
  final context = rootNavigatorKey.currentContext;
  if (context == null) return;
  final String title, body;
  if (result == ServerCompatibility.serverTooOld) {
    title = l10n.serverTooOldTitle;
    body = l10n.serverTooOldBody;
  } else {
    title = l10n.clientTooOldTitle;
    body = l10n.clientTooOldBody;
  }
  unawaited(
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(l10n.ok),
          ),
        ],
      ),
    ),
  );
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
    warnOnIncompatibleServer(server);
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
      logger.i("no saved account, redirect to official server login");
      await connectToOfficialServer(ref);
      if (context.mounted) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (context) => Auth()),
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
      logger.w(
        "auto-login: no account succeeded, redirect to official server login",
      );
      await connectToOfficialServer(ref);
      if (context.mounted) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (context) => Auth()),
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
