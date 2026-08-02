import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/session/session_record.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';

import 'helpers/live_client_fixture.dart';

/// Full UI-level E2E: pumps the REAL [SessionTab] widget tree against a live
/// server, drives real interactions (type → tap send → long-press → quote →
/// send reply), and asserts on rendered widgets + Riverpod state.
///
/// This is the only layer that catches widget-wiring bugs:
/// - Does [SessionTab]'s send button correctly read [quoteTargetProvider] and
///   inject `quoteMsgId` into the `SendMsgRequest`?
/// - Does [SessionNotifier.receiveMsg] → state rebuild → [MessageWidget]
///   actually render the quote block from real parsed data?
///
/// Note: [SessionTab] must be mounted BEFORE modifying [sessionProvider] /
/// [quoteTargetProvider] state — these are auto-dispose providers that need
/// a live `ref.watch` from the widget to stay alive.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final fx = LiveClientFixture();

  tearDownAll(() async => await fx.tearDown());

  testWidgets(
    'UI: send → long-press → quote → reply renders quote block from live data',
    (tester) async {
      final ok = await fx.setUp();
      if (!ok) {
        markTestSkipped('server at localhost:7777 is not reachable');
        return;
      }

      final bob = await fx.app.registerUser();
      final sid = await fx.user.createSession([bob]).then((r) => r.sessionId);

      final container = fx.container;

      // Pump SessionTab FIRST so its ref.watch keeps auto-dispose
      // providers (sessionProvider, quoteTargetProvider) alive.
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: const Scaffold(body: SessionTab()),
          ),
        ),
      );
      await tester.pump();

      // NOW safe to modify state (widget is watching these providers).
      container
          .read(sessionProvider.notifier)
          .openSessionTab(sid, 'Test Session');
      await tester.pump();

      // Register the msg listener (normally done by SessionList.initState).
      final eventSystem = container.read(ourChatEventSystemProvider.notifier);
      void onMsg(UserMsg m) =>
          container.read(sessionProvider.notifier).receiveMsg(m);
      eventSystem.addListener(FetchMsgsResponse_RespondEventType.msg, onMsg);

      Future<void> uiSend(String text) async {
        final tf = tester.widget<TextField>(find.byType(TextField));
        tf.controller!.text = text;
        container.read(inputTextProvider.notifier).setText(text);
        await tester.pump(const Duration(milliseconds: 300));
        await tester.tap(find.byIcon(Icons.send));
        await tester.pump(const Duration(milliseconds: 300));
      }

      // ── ① Send original message via real UI ──
      await uiSend('original from UI');
      await _pumpUntil(
        tester,
        () => container
            .read(sessionProvider)
            .currentSessionRecords
            .any((m) => m.markdownText == 'original from UI'),
      );

      final original = container
          .read(sessionProvider)
          .currentSessionRecords
          .firstWhere((m) => m.markdownText == 'original from UI');
      expect(original.quoteMsgId, isNull, reason: 'original has no quote');

      // ── ② Long-press the message → tap "引用" ──
      await tester.longPress(find.byType(MessageWidget).first);
      await tester.pumpAndSettle();
      await tester.tap(find.text(l10n.quote));
      await tester.pumpAndSettle();
      expect(
        container.read(quoteTargetProvider)?.markdownText,
        'original from UI',
        reason: 'quote target should be set to the original message',
      );

      // ── ③ Send reply via real UI (SessionTab injects quoteMsgId) ──
      await uiSend('reply from UI');
      await _pumpUntil(
        tester,
        () => container
            .read(sessionProvider)
            .currentSessionRecords
            .any((m) => m.markdownText == 'reply from UI'),
      );

      // ── ④ Assert: reply's quote fields parsed correctly by real client code ──
      final reply = container
          .read(sessionProvider)
          .currentSessionRecords
          .firstWhere((m) => m.markdownText == 'reply from UI');
      expect(reply.quoteMarkdownText, 'original from UI');
      expect(reply.quoteMsgId, original.eventId);
      expect(reply.quoteSenderId, fx.accountId);

      // ── ⑤ Quote target was cleared after send ──
      expect(container.read(quoteTargetProvider), isNull);

      // ── ⑥ Quote block rendered in the widget tree ──
      expect(
        find.text('original from UI'),
        findsWidgets,
        reason: 'quote block should render the quoted text as a Text widget',
      );

      // Cleanup
      eventSystem.removeListener(FetchMsgsResponse_RespondEventType.msg, onMsg);
      try {
        await fx.user.deleteSession(sid);
      } catch (_) {}
    },
  );
}

/// Repeatedly pump frames until [condition] is true (or timeout).
Future<void> _pumpUntil(
  WidgetTester tester,
  bool Function() condition, {
  Duration timeout = const Duration(seconds: 15),
}) async {
  final end = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(end)) {
    await tester.pump(const Duration(milliseconds: 200));
    if (condition()) return;
  }
  fail('_pumpUntil: condition not met within $timeout');
}
