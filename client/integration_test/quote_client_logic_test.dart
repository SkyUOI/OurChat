import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'helpers/live_client_fixture.dart';

/// Exercises REAL client app logic against a live server:
///
/// - [LiveClientFixture.sendAsClient] → calls the real [UserMsg.send] (send
///   path + E2EE decision + request packaging).
/// - [LiveClientFixture.waitForMsg] → receives via the real
///   [OurChatEventSystem.listenEvents] pipeline (stream parsing +
///   [UserMsg.quoteFieldsFromMsg] + drift persistence + listener dispatch).
///
/// This catches bugs that the raw-gRPC contract test
/// ([quote_flow_test]) cannot: wrong proto field indices,
/// missing `Int64.ZERO→null` conversion, broken stream→DB wiring, etc.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final fx = LiveClientFixture();

  tearDownAll(() async => await fx.tearDown());

  testWidgets(
    'client send path injects quoteMsgId and receive path parses quote fields',
    (_) async {
      final ok = await fx.setUp();
      if (!ok) {
        markTestSkipped('server at localhost:7777 is not reachable');
        return;
      }

      // Provision a peer + session via raw stubs (setup only).
      final bob = await fx.app.registerUser();
      final sessionId = await fx.user
          .createSession([bob])
          .then((r) => r.sessionId);

      // ── ① REAL send path: UserMsg.send() sends the original message ──
      await fx.sendAsClient(
        sessionId: sessionId,
        markdownText: 'original message',
      );

      // ── ② REAL receive path: OurChatEventSystem delivers the parsed UserMsg ──
      final original = await fx.waitForMsg(
        (m) => m.markdownText == 'original message',
      );
      expect(original.eventId, isNotNull);
      expect(original.quoteMsgId, isNull, reason: 'original has no quote');
      expect(original.quoteMarkdownText, isEmpty);

      // ── ③ REAL send path with quote: UserMsg.send injects quoteMsgId ──
      await fx.sendAsClient(
        sessionId: sessionId,
        markdownText: 'quoted reply',
        quoteMsgId: original.eventId,
      );

      // ── ④ REAL receive path: quoteFieldsFromMsg parses the wire proto ──
      final reply = await fx.waitForMsg(
        (m) => m.markdownText == 'quoted reply',
      );

      // These assertions verify the DART-SPECIFIC parsing logic
      // (quoteFieldsFromMsg) — the part Rust tests never touch.
      expect(reply.quoteMsgId, original.eventId);
      expect(
        reply.quoteSenderId,
        fx.accountId,
        reason: 'original sender is the logged-in account',
      );
      expect(reply.quoteMarkdownText, 'original message');
      expect(reply.quoteInvolvedFiles, isEmpty);

      // ── ⑤ DB assertion: real drift Record row was persisted ──
      final records = await fx.dbRecords();
      final replyRow = records.firstWhere(
        (r) =>
            (jsonDecode(r.data) as Map<String, dynamic>)['markdown_text'] ==
            'quoted reply',
      );
      final rowData = jsonDecode(replyRow.data) as Map<String, dynamic>;
      expect(rowData['quote_markdown_text'], 'original message');
      expect(rowData['quote_msg_id'], original.eventId!.toInt());
      expect(rowData['quote_sender_id'], fx.accountId.toInt());

      // Cleanup
      try {
        await fx.user.deleteSession(sessionId);
      } catch (_) {}
    },
  );
}
