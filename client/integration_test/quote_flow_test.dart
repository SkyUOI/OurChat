import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'helpers/oc_test_app.dart';

/// Contract smoke test: verifies the server + proto contract for quote
/// messages via raw gRPC stubs. For a test that exercises the client's own
/// parsing/send code, see [quote_client_logic_test].
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('quote message round-trips through the live server', (_) async {
    final app = OcTestApp('localhost', 7777, false);
    if (!await app.probe()) {
      markTestSkipped('server at localhost:7777 is not reachable');
      return;
    }

    final alice = await app.registerUser();
    final bob = await app.registerUser();
    final sessionId = await app.createSession(alice, [bob]);
    expect(sessionId, isNot(Int64.ZERO));

    final original = await alice.sendMsg(sessionId, 'original message');
    expect(original.msgId, isNot(Int64.ZERO));

    await alice.sendMsgWithQuote(sessionId, 'quoted reply', original.msgId);

    final msgs = await alice.fetchMsgs().fetchUntil(
      (m) => m.markdownText == 'quoted reply',
    );
    final reply = msgs.firstWhere((m) => m.markdownText == 'quoted reply');
    expect(reply.quoteMsgId, original.msgId);
    expect(reply.quoteSenderId, alice.id);
    expect(reply.quoteMarkdownText, 'original message');
    expect(reply.quoteInvolvedFiles, isEmpty);

    final originalMsg = msgs.firstWhere(
      (m) => m.markdownText == 'original message',
      orElse: () => throw StateError('original message not found'),
    );
    expect(originalMsg.quoteMsgId, Int64.ZERO);

    try {
      await alice.deleteSession(sessionId);
    } catch (_) {}
    await app.dispose();
  });
}
