import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:ourchat/core/event.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

// Platform-conditional implementation: the real flutter_local_notifications
// wrapper on desktop/mobile, and the browser Notification API wrapper on web
// (same conditional import pattern as window_manager / tray_manager in
// main.dart).
import 'package:ourchat/core/notification.dart'
    if (dart.library.html) 'package:ourchat/core/notification_web.dart';

export 'package:ourchat/core/notification.dart'
    if (dart.library.html) 'package:ourchat/core/notification_web.dart';

part 'notification_service.g.dart';

/// Provider for the cross-platform system notification service. KeepAlive
/// because the service holds the plugin initialization state for the whole
/// app lifetime. Override with a fake in tests to avoid touching platform
/// plugins.
@Riverpod(keepAlive: true)
OurChatNotificationService ourChatNotificationService(Ref ref) {
  return OurChatNotificationService();
}

/// Decide whether an incoming message deserves a user-visible attention
/// signal (system notification + tray flash) — issue #199.
///
/// Notify when the receiving conversation is NOT the one currently open, or
/// when the app is backgrounded (even an open conversation cannot be seen
/// then). Messages synced back from our own account (multi-device echo) never
/// notify.
@visibleForTesting
bool shouldNotifyNewMessage({
  required Int64? msgSessionId,
  required Int64? currentSessionId,
  required bool appInForeground,
  Int64? senderId,
  Int64? thisAccountId,
}) {
  if (senderId != null && thisAccountId != null && senderId == thisAccountId) {
    return false;
  }
  final viewingThisSession =
      msgSessionId != null && msgSessionId == currentSessionId;
  if (viewingThisSession && appInForeground) {
    return false;
  }
  return true;
}

/// Body text for a new-message notification, honoring the "show message
/// content in notifications" privacy switch: when [showContent] is false the
/// body is always the generic [genericBody] ("New message") regardless of the
/// actual [preview].
@visibleForTesting
String newMessageNotificationBody({
  required bool showContent,
  required String preview,
  required String genericBody,
}) {
  if (!showContent) return genericBody;
  final trimmed = preview.trim();
  return trimmed.isEmpty ? genericBody : trimmed;
}

/// Run the full new-message attention pipeline for [msg]: decide whether to
/// notify ([shouldNotifyNewMessage]), flash the tray (only when [flashTray]
/// is provided — desktop only), and show a system notification via
/// [notificationService] with a body built by [newMessageNotificationBody].
///
/// Everything is passed in explicitly (no provider reads) so the trigger
/// logic can be unit tested with a fake notification service and a recording
/// tray-flash callback (issue #199).
Future<void> notifyNewMessage(
  UserMsg msg, {
  required OurChatNotificationService notificationService,
  required Int64? currentSessionId,
  required bool appInForeground,
  Int64? thisAccountId,
  required bool showContent,
  required String title,
  required String preview,
  required String genericBody,
  void Function()? flashTray,
}) async {
  if (!shouldNotifyNewMessage(
    msgSessionId: msg.sessionId,
    currentSessionId: currentSessionId,
    appInForeground: appInForeground,
    senderId: msg.senderId,
    thisAccountId: thisAccountId,
  )) {
    return;
  }
  if (flashTray != null) {
    flashTray();
  }
  await notificationService.showNewMessage(
    title: title,
    body: newMessageNotificationBody(
      showContent: showContent,
      preview: preview,
      genericBody: genericBody,
    ),
    payload: msg.sessionId?.toString(),
  );
}
