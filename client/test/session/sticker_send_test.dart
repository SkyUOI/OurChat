import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/e2ee.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/session/sticker_panel.dart';
import 'test_harness.dart';

/// The sticker send path (issue #147): tapping a sticker in the panel funnels
/// into sendStickerMessage, which must emit an image message referencing the
/// already-uploaded file through involvedFiles and an io:// markdown image.
void main() {
  setUpAll(() {
    registerFallbackValue(SendMsgRequest());
  });

  test('sendStickerMessage sends an image message referencing the file key',
      () async {
    final client = MockOurChatClient();
    when(
      () => client.sendMsg(any(), options: any(named: 'options')),
    ).thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
      ],
    );
    addTearDown(container.dispose);

    final server = container.read(ourChatServerProvider);
    final store = container.read(
      e2eeStoreProvider(testServerId, Int64(1)).notifier,
    );
    final sent = await sendStickerMessage(
      server: server,
      e2eeStore: store,
      sessionId: Int64(1),
      fileKey: 'stk-1',
    );

    expect(sent, isTrue);
    final request = verify(
      () => client.sendMsg(captureAny(), options: any(named: 'options')),
    ).captured.single as SendMsgRequest;
    expect(request.sessionId, Int64(1));
    expect(request.markdownText, '![sticker](io://0)');
    expect(request.involvedFiles, ['stk-1']);
    expect(request.isEncrypted, isFalse);
  });
}
