import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/download/v1/download.pb.dart';
import 'package:ourchat/service/ourchat/sticker/v1/sticker.pb.dart';
import 'package:ourchat/session/emoji_panel.dart';
import 'package:ourchat/session/session_record.dart';
import 'package:ourchat/session/sticker_panel.dart';
import 'test_harness.dart';

void main() {
  setUpAll(() {
    registerFallbackValue(AddStickerRequest());
    registerFallbackValue(GetStickersRequest());
    registerFallbackValue(DownloadRequest());
  });

  group('StickerPanel (issue #147)', () {
    testWidgets('renders the empty hint when the collection is empty', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.getStickers(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(GetStickersResponse()));

      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: StickerPanel(onStickerSelected: (_) {}),
        ),
      );
      await tester.pumpAndSettle(const Duration(seconds: 1));

      expect(find.text(l10n.stickersEmpty), findsOneWidget);
    });
  });

  group('EmojiPanel tabs (issue #147)', () {
    testWidgets('without a sticker callback the panel stays emoji-only', (
      tester,
    ) async {
      final container = ProviderContainer(
        overrides: [activeAccountTestOverride],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: EmojiPanel(onEmojiSelected: (_) {}),
        ),
      );

      expect(find.byType(TabBar), findsNothing);
      // Emoji content is rendered directly.
      expect(find.text('😀'), findsOneWidget);
    });

    testWidgets('with a sticker callback an emoji tab and a stickers tab show',
        (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.getStickers(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(GetStickersResponse()));

      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: EmojiPanel(
            onEmojiSelected: (_) {},
            onStickerSelected: (_) {},
          ),
        ),
      );

      expect(find.byType(TabBar), findsOneWidget);
      expect(find.text(l10n.emoji), findsOneWidget);
      expect(find.text(l10n.stickersTab), findsOneWidget);
    });
  });

  group('message menu "Save as sticker" (issue #147)', () {
    Future<ProviderContainer> pumpMessage(
      WidgetTester tester,
      MockOurChatClient client,
      UserMsg msg,
    ) async {
      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
          overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
        ],
      );
      addTearDown(container.dispose);
      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: MessageWidget(msg: msg, opacity: 1.0),
        ),
      );
      await tester.pump();
      return container;
    }

    /// The selectable markdown text competes for the long-press gesture, so
    /// press an empty spot inside the message container instead.
    Future<void> longPressMessage(WidgetTester tester) async {
      final rect = tester.getRect(find.byType(MessageWidget));
      await tester.longPressAt(rect.bottomRight - const Offset(10, 10));
      // Fixed pumps (not pumpAndSettle): an image message's loading spinner
      // animates forever and would time the settle out.
      await tester.pump(const Duration(seconds: 1));
      await tester.pump();
    }

    testWidgets('a message with files offers saving its first file', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.addSticker(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(AddStickerResponse()));
      // The io:// image preview would otherwise spin forever in tests.
      when(
        () => client.download(any(), options: any(named: 'options')),
      ).thenThrow(GrpcError.notFound('no file in test'));

      await pumpMessage(
        tester,
        client,
        UserMsg(
          senderId: Int64(1),
          eventId: Int64(10),
          markdownText: '![img](io://0)',
          involvedFiles: const ['key-1', 'key-2'],
          sessionId: Int64(1),
        ),
      );

      await longPressMessage(tester);
      expect(find.text(l10n.saveSticker), findsOneWidget);

      await tester.tap(find.text(l10n.saveSticker));
      // Fixed pumps: the message-area image spinner animates forever.
      await tester.pump(const Duration(seconds: 1));
      await tester.pump();

      final request = verify(
        () => client.addSticker(
          captureAny(),
          options: any(named: 'options'),
        ),
      ).captured.single as AddStickerRequest;
      expect(request.fileKey, 'key-1');
    });

    testWidgets('a plain text message offers no sticker entry', (
      tester,
    ) async {
      await pumpMessage(
        tester,
        MockOurChatClient(),
        UserMsg(
          senderId: Int64(1),
          eventId: Int64(11),
          markdownText: 'plain text',
          sessionId: Int64(1),
        ),
      );

      await longPressMessage(tester);

      expect(find.text(l10n.saveSticker), findsNothing);
      expect(find.text(l10n.quote), findsOneWidget);
    });
  });
}
