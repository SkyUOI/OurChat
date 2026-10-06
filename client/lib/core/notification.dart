import 'dart:io' show Platform;

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:ourchat/core/log.dart';

/// System notification service for desktop and mobile, backed by
/// `flutter_local_notifications` (issue #199).
///
/// This file is only compiled for non-web targets: the neutral entry point
/// (`core/notification_service.dart`) swaps in `stubs/notification_stub.dart`
/// on the web, mirroring the `window_manager`/`tray_manager` conditional
/// import pattern.
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
  /// flooding the notification center.
  static const int _newMessageNotificationId = 1;

  /// Stable identity used by the Windows toast implementation. A constant AUMD
  /// + GUID keeps toasts attributable to the app across runs.
  static const String _windowsAppUserModelId = 'org.skyuoi.ourchat';
  static const String _windowsGuid = '8f0d0d5e-6d9e-4b1f-9d3a-1c4b5a6e7d80';

  /// Initialize the plugin and, where the platform needs it, request
  /// permission. Best effort: failures are logged and leave the service
  /// disabled instead of breaking messaging.
  Future<void> init() async {
    if (_initialized) return;
    try {
      const android = AndroidInitializationSettings('@mipmap/ic_launcher');
      const darwin = DarwinInitializationSettings();
      const linux = LinuxInitializationSettings(defaultActionName: 'open');
      const windows = WindowsInitializationSettings(
        appName: 'OurChat',
        appUserModelId: _windowsAppUserModelId,
        guid: _windowsGuid,
      );
      const settings = InitializationSettings(
        android: android,
        iOS: darwin,
        macOS: darwin,
        linux: linux,
        windows: windows,
      );
      await _plugin.initialize(
        settings: settings,
        onDidReceiveNotificationResponse: _handleNotificationResponse,
      );
      if (Platform.isAndroid) {
        // Android 13+ requires the runtime POST_NOTIFICATIONS permission.
        await _plugin
            .resolvePlatformSpecificImplementation<
              AndroidFlutterLocalNotificationsPlugin
            >()
            ?.requestNotificationsPermission();
      }
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
      const androidDetails = AndroidNotificationDetails(
        'new_message',
        'OurChat',
        importance: Importance.max,
        priority: Priority.high,
        category: AndroidNotificationCategory.message,
      );
      const details = NotificationDetails(
        // Linux/Windows fall back to their default presentation; iOS/macOS
        // inherit the permissions granted during init.
        android: androidDetails,
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
    // Best effort: bring the app back to the foreground (the actual
    // show/stop-flash logic lives in the callback wired by the app shell).
    // Deep-linking to the conversation carried by the payload is not wired
    // up yet (TODO #199).
    onNotificationTap?.call(response.payload);
  }
}
