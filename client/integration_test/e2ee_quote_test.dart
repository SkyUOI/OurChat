import 'dart:convert';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/e2ee.dart';
import 'package:ourchat/service/ourchat/session/e2eeize_and_dee2eeize_session/v1/e2eeize_and_dee2eeize_session.pb.dart';

import 'helpers/live_client_fixture.dart';

/// E2EE + Quote interaction test: verifies that [UserMsg.send] properly
/// encrypts messages in E2EE sessions (BUG #4 fix), and that quoting an
/// encrypted message leaves `quote_markdown_text` empty on the wire (server
/// can't read E2EE content), while the local DB retains the decrypted text
/// for [MessageWidget._resolveQuote] to use.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final fx = LiveClientFixture();

  tearDownAll(() async => await fx.tearDown());

  testWidgets('E2EE message is encrypted on wire + quote preserves fields', (
    _,
  ) async {
    final ok = await fx.setUp();
    if (!ok) {
      markTestSkipped('server at localhost:7777 is not reachable');
      return;
    }

    final bob = await fx.app.registerUser();
    final sid = await fx.user.createSession([bob]).then((r) => r.sessionId);

    // ── ① E2eeize the session ──
    await fx.user.stub.e2eeizeSession(E2eeizeSessionRequest(sessionId: sid));

    // Wait for the room key exchange (updateRoomKey event processed by the
    // real OurChatEventSystem).
    final container = fx.container;
    for (int i = 0; i < 50; i++) {
      if (container
          .read(e2eeStoreProvider(fx.serverId, fx.accountId).notifier)
          .hasKey(sid)) {
        break;
      }
      await Future.delayed(const Duration(milliseconds: 200));
    }
    expect(
      container
          .read(e2eeStoreProvider(fx.serverId, fx.accountId).notifier)
          .hasKey(sid),
      isTrue,
      reason: 'room key should be stored after e2eeizeSession',
    );

    // ── ② Send + immediately wait for decrypted delivery ──
    // MUST register waitForMsg listener right after send, before any other
    // async work, so the event system's listener doesn't miss the message.
    await fx.sendAsClient(sessionId: sid, markdownText: 'secret message');
    final decrypted = await fx.waitForMsg(
      (m) => m.markdownText == 'secret message',
    );
    expect(
      decrypted.markdownText,
      'secret message',
      reason: 'event system should decrypt using the room key',
    );
    expect(decrypted.eventId, isNotNull);
    expect(
      decrypted.eventId,
      isNot(equals(Int64.ZERO)),
      reason: 'server should assign a non-zero event id',
    );

    // ── ③ Verify the wire-level message was encrypted ──
    final rawMsgs = await fx.user
        .fetchMsgs(historyLimit: Int64(10))
        .fetchUntil((m) => m.sessionId == sid && m.isEncrypted);
    final rawSecret = rawMsgs.firstWhere((m) => m.isEncrypted);
    expect(
      rawSecret.isEncrypted,
      isTrue,
      reason: 'BUG #4: message must be encrypted on the wire in E2EE sessions',
    );
    expect(
      rawSecret.markdownText,
      isNot(equals('secret message')),
      reason: 'wire text should be ciphertext',
    );

    // ── ④ Send a quote of the encrypted message ──
    await fx.sendAsClient(
      sessionId: sid,
      markdownText: 'encrypted quote reply',
      quoteMsgId: decrypted.eventId,
    );
    final reply = await fx.waitForMsg(
      (m) => m.markdownText == 'encrypted quote reply',
    );

    // ── ⑤ Verify E2EE quote behavior ──
    // Server fills in ID + sender (from metadata), but NOT the text
    // (it can't decrypt the quoted message).
    expect(
      reply.quoteMsgId,
      decrypted.eventId,
      reason: 'server fills in quote_msg_id from message metadata',
    );
    expect(
      reply.quoteSenderId,
      fx.accountId,
      reason: 'server fills in quote_sender_id from message metadata',
    );

    // ── ⑥ Verify local DB retains decrypted text for _resolveQuote ──
    final records = await fx.dbRecords();
    final originalRow = records.firstWhere(
      (r) => r.eventId == BigInt.from(decrypted.eventId!.toInt()),
    );
    final originalData = jsonDecode(originalRow.data) as Map<String, dynamic>;
    expect(
      originalData['markdown_text'],
      'secret message',
      reason: 'local DB should have decrypted text for _resolveQuote',
    );

    // Cleanup
    try {
      await fx.user.deleteSession(sid);
    } catch (_) {}
  });
}
