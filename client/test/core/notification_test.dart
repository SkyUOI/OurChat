import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/notification_service.dart';
import 'package:ourchat/core/stubs/notification_stub.dart' as web_stub;

/// Records every call made to the notification service so tests can assert
/// on the trigger logic without touching platform plugins.
class RecordingNotificationService implements OurChatNotificationService {
  final messages = <({String title, String body, String? payload})>[];
  int cancelAllCalls = 0;

  @override
  void Function(String? payload)? onNotificationTap;

  @override
  Future<void> init() async {}

  @override
  Future<void> showNewMessage({
    required String title,
    required String body,
    String? payload,
  }) async {
    messages.add((title: title, body: body, payload: payload));
  }

  @override
  Future<void> cancelAll() async {
    cancelAllCalls++;
  }
}

UserMsg buildMsg({
  Int64? sessionId,
  Int64? senderId,
  String text = 'hello **world**',
}) {
  return UserMsg(
    eventId: Int64(1),
    senderId: senderId ?? Int64(7),
    sessionId: sessionId ?? Int64(200),
    markdownText: text,
  );
}

/// Tests for the new-message system notification feature (issue #199):
/// the web stub, the pure trigger decision, the notification-body privacy
/// switch, and the notification pipeline with an injected fake service.
void main() {
  group('web notification stub', () {
    test('can be instantiated and every method is a safe no-op', () async {
      final svc = web_stub.OurChatNotificationService();
      expect(svc.onNotificationTap, isNull);
      // None of these may throw (no plugins exist on the web build).
      await svc.init();
      await svc.showNewMessage(title: 't', body: 'b', payload: 'p');
      await svc.cancelAll();
    });
  });

  group('shouldNotifyNewMessage', () {
    test('notifies for a session that is not currently open', () {
      expect(
        shouldNotifyNewMessage(
          msgSessionId: Int64(2),
          currentSessionId: Int64(1),
          appInForeground: true,
        ),
        isTrue,
      );
    });

    test(
      'stays silent for the open session while the app is in foreground',
      () {
        expect(
          shouldNotifyNewMessage(
            msgSessionId: Int64(1),
            currentSessionId: Int64(1),
            appInForeground: true,
          ),
          isFalse,
        );
      },
    );

    test('notifies for the open session while the app is backgrounded', () {
      expect(
        shouldNotifyNewMessage(
          msgSessionId: Int64(1),
          currentSessionId: Int64(1),
          appInForeground: false,
        ),
        isTrue,
      );
    });

    test('never notifies for own multi-device echo', () {
      expect(
        shouldNotifyNewMessage(
          msgSessionId: Int64(1),
          currentSessionId: Int64(2),
          appInForeground: true,
          senderId: Int64(42),
          thisAccountId: Int64(42),
        ),
        isFalse,
      );
    });
  });

  group('newMessageNotificationBody (privacy switch)', () {
    test('content shown: the preview text is used', () {
      expect(
        newMessageNotificationBody(
          showContent: true,
          preview: 'hello world',
          genericBody: 'New message',
        ),
        'hello world',
      );
    });

    test(
      'content hidden: the body is the generic text regardless of preview',
      () {
        expect(
          newMessageNotificationBody(
            showContent: false,
            preview: 'secret content',
            genericBody: 'New message',
          ),
          'New message',
        );
      },
    );

    test('empty preview falls back to the generic text', () {
      expect(
        newMessageNotificationBody(
          showContent: true,
          preview: '   ',
          genericBody: 'New message',
        ),
        'New message',
      );
    });
  });

  group('notifyNewMessage pipeline (fake service + injected tray flash)', () {
    test(
      'non-current session: shows notification and flashes the tray',
      () async {
        final svc = RecordingNotificationService();
        var flashCalls = 0;
        await notifyNewMessage(
          buildMsg(sessionId: Int64(200), senderId: Int64(7)),
          notificationService: svc,
          currentSessionId: Int64(100),
          appInForeground: true,
          thisAccountId: Int64(1),
          showContent: true,
          title: 'Dev chat',
          preview: 'hello world',
          genericBody: 'New message',
          flashTray: () => flashCalls++,
        );
        expect(flashCalls, 1);
        expect(svc.messages, hasLength(1));
        expect(svc.messages.single.title, 'Dev chat');
        expect(svc.messages.single.body, 'hello world');
        expect(svc.messages.single.payload, '200');
      },
    );

    test(
      'currently open session in foreground: no notification, no flash',
      () async {
        final svc = RecordingNotificationService();
        var flashCalls = 0;
        await notifyNewMessage(
          buildMsg(sessionId: Int64(100), senderId: Int64(7)),
          notificationService: svc,
          currentSessionId: Int64(100),
          appInForeground: true,
          thisAccountId: Int64(1),
          showContent: true,
          title: 'Dev chat',
          preview: 'hello world',
          genericBody: 'New message',
          flashTray: () => flashCalls++,
        );
        expect(flashCalls, 0);
        expect(svc.messages, isEmpty);
      },
    );

    test('open session but app backgrounded: still notifies', () async {
      final svc = RecordingNotificationService();
      await notifyNewMessage(
        buildMsg(sessionId: Int64(100), senderId: Int64(7)),
        notificationService: svc,
        currentSessionId: Int64(100),
        appInForeground: false,
        thisAccountId: Int64(1),
        showContent: true,
        title: 'Dev chat',
        preview: 'hello world',
        genericBody: 'New message',
      );
      expect(svc.messages, hasLength(1));
    });

    test(
      'privacy off: body is the generic text, notification still shown',
      () async {
        final svc = RecordingNotificationService();
        await notifyNewMessage(
          buildMsg(sessionId: Int64(200), senderId: Int64(7)),
          notificationService: svc,
          currentSessionId: Int64(100),
          appInForeground: true,
          thisAccountId: Int64(1),
          showContent: false,
          title: 'Dev chat',
          preview: 'hello world',
          genericBody: 'New message',
        );
        expect(svc.messages, hasLength(1));
        expect(svc.messages.single.title, 'Dev chat');
        expect(svc.messages.single.body, 'New message');
      },
    );

    test('no flashTray injected (non-desktop platforms): no crash, still '
        'notifies', () async {
      final svc = RecordingNotificationService();
      await notifyNewMessage(
        buildMsg(sessionId: Int64(200), senderId: Int64(7)),
        notificationService: svc,
        currentSessionId: Int64(100),
        appInForeground: true,
        thisAccountId: Int64(1),
        showContent: true,
        title: 'Dev chat',
        preview: 'hello world',
        genericBody: 'New message',
      );
      expect(svc.messages, hasLength(1));
    });

    test('message from this account (multi-device echo): silent', () async {
      final svc = RecordingNotificationService();
      var flashCalls = 0;
      await notifyNewMessage(
        buildMsg(sessionId: Int64(200), senderId: Int64(1)),
        notificationService: svc,
        currentSessionId: Int64(100),
        appInForeground: true,
        thisAccountId: Int64(1),
        showContent: true,
        title: 'Dev chat',
        preview: 'hello world',
        genericBody: 'New message',
        flashTray: () => flashCalls++,
      );
      expect(flashCalls, 0);
      expect(svc.messages, isEmpty);
    });
  });
}
