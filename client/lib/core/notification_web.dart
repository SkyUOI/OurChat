import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:ourchat/core/log.dart';

/// System notification service for the web, backed by the browser
/// Notification API through `flutter_local_notifications`' web
/// implementation (issue #199).
///
/// The web plugin registers its own service worker at runtime and needs the
/// Notification permission before showing anything; `init()` does both on a
/// best-effort basis. When the browser denies the permission or does not
/// support notifications/service workers at all, the service degrades to a
/// no-op instead of breaking messaging.
class OurChatNotificationService {
  final FlutterLocalNotificationsPlugin _plugin =
      FlutterLocalNotificationsPlugin();

  /// Invoked when the user taps a notification. Wired by the app shell (see
  /// `_MainAppState.initState` in `main.dart`) so this file stays free of
  /// window/tray dependencies.
  void Function(String? payload)? onNotificationTap;

  bool _initialized = false;

  /// Notification id used for new-message notifications. Reusing a single id
  /// means a burst of messages replaces the previous notification instead of
  /// flooding the notification center (on web the id maps to the
  /// notification tag).
  static const int _newMessageNotificationId = 1;

  /// App icon shown next to the notification (resolved against the site
  /// root, where `flutter build web` emits the PWA icons).
  static const String _iconUrl = 'icons/Icon-192.png';

  /// Initialize the plugin (registers the service worker) and request the
  /// Notification permission. Best effort: failures are logged and leave the
  /// service disabled instead of breaking messaging.
  Future<void> init() async {
    if (_initialized) return;
    try {
      const settings = InitializationSettings(web: WebInitializationSettings());
      final ok = await _plugin.initialize(
        settings: settings,
        onDidReceiveNotificationResponse: _handleNotificationResponse,
      );
      if (ok != true) return;
      // Browsers only show notifications once the user grants the
      // Notification permission. The request is best-effort here: init is
      // not tied to a user gesture, so some browsers may auto-deny — show()
      // then fails and is swallowed below.
      await _plugin
          .resolvePlatformSpecificImplementation<
            WebFlutterLocalNotificationsPlugin
          >()
          ?.requestNotificationsPermission();
      _initialized = true;
    } catch (e) {
      logger.w('notifications: init failed: $e');
    }
  }

  /// Show a "new message" notification. [title] is usually the conversation
  /// (or sender) name; [body] is a message preview or a generic placeholder
  /// when the content privacy switch is off. [payload] round-trips to
  /// [onNotificationTap] when the notification is tapped.
  Future<void> showNewMessage({
    required String title,
    required String body,
    String? payload,
  }) async {
    if (!_initialized) {
      await init();
    }
    if (!_initialized) return;
    try {
      final details = NotificationDetails(
        web: WebNotificationDetails(iconUrl: Uri.parse(_iconUrl)),
      );
      await _plugin.show(
        id: _newMessageNotificationId,
        title: title,
        body: body,
        notificationDetails: details,
        payload: payload,
      );
    } catch (e) {
      logger.w('notifications: show failed: $e');
    }
  }

  /// Remove every notification previously shown by this app.
  Future<void> cancelAll() async {
    if (!_initialized) return;
    try {
      await _plugin.cancelAll();
    } catch (e) {
      logger.w('notifications: cancelAll failed: $e');
    }
  }

  void _handleNotificationResponse(NotificationResponse response) {
    // Best effort: the shell callback stops the tray flash / focuses the
    // window (no-op on web). Deep-linking to the conversation carried by the
    // payload is not wired up yet (TODO #199).
    onNotificationTap?.call(response.payload);
  }
}
