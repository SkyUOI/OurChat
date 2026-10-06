// Stub implementation of the system notification service for the web
// platform. Mirrors the API of `core/notification.dart`; selected by the
// conditional import in `core/notification_service.dart` (same pattern as
// `stubs/window_manager_stub.dart`).
//
// The web build does not integrate a notification plugin (browser
// notification support may be added later, see issue #199).
class OurChatNotificationService {
  /// Never called on web (no notifications are shown).
  void Function(String? payload)? onNotificationTap;

  Future<void> init() async {
    // No-op on web
  }

  Future<void> showNewMessage({
    required String title,
    required String body,
    String? payload,
  }) async {
    // No-op on web
  }

  Future<void> cancelAll() async {
    // No-op on web
  }
}
