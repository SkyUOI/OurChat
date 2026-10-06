import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/emoji_panel.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'test_harness.dart';

void main() {
  setUpAll(() {
    registerFallbackValue(SendMsgRequest());
  });

  testWidgets(
    'emoji button toggles the panel and tapping an emoji updates the draft',
    (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.sendMsg(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
          overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
        ],
      );
      addTearDown(container.dispose);

      container.read(sessionProvider.notifier).state = SessionState(
        tabIndex: TabType.session,
        currentSessionId: Int64(1),
        currentSessionRecords: const [],
      );

      await tester.pumpWidget(
        buildTestApp(container: container, child: const SessionTab()),
      );
      await tester.pump();

      expect(find.byType(EmojiPanel), findsNothing);

      await tester.tap(find.byIcon(Icons.emoji_emotions));
      await tester.pumpAndSettle();
      expect(find.byType(EmojiPanel), findsOneWidget);

      final emojiFinder = find.descendant(
        of: find.byType(EmojiPanel),
        matching: find.text('😀'),
      );
      expect(emojiFinder, findsOneWidget);
      await tester.tap(emojiFinder);
      await tester.pump();

      expect(container.read(inputTextProvider), '😀');
      final field = tester.widget<TextFormField>(find.byType(TextFormField));
      expect(field.controller!.text, '😀');

      // Sending the message collapses the panel again.
      await tester.tap(find.text(l10n.send));
      await tester.pumpAndSettle();
      expect(find.byType(EmojiPanel), findsNothing);
      expect(container.read(inputTextProvider), '');
    },
  );

  testWidgets('several emoji can be inserted in a row and categories switch', (
    tester,
  ) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(
          FakeOurChatServer(MockOurChatClient()),
        ),
      ],
    );
    addTearDown(container.dispose);

    container.read(sessionProvider.notifier).state = SessionState(
      tabIndex: TabType.session,
      currentSessionId: Int64(1),
      currentSessionRecords: const [],
    );

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    await tester.tap(find.byIcon(Icons.emoji_emotions));
    await tester.pumpAndSettle();

    // Two taps in a row keep the panel open and append both emoji.
    Future<void> tapEmoji(String emoji) async {
      await tester.tap(
        find.descendant(
          of: find.byType(EmojiPanel),
          matching: find.text(emoji),
        ),
      );
      await tester.pump();
    }

    await tapEmoji('😀');
    await tapEmoji('😃');
    expect(container.read(inputTextProvider), '😀😃');
    expect(find.byType(EmojiPanel), findsOneWidget);

    // Switch to the people category and pick from there.
    await tester.tap(find.text(l10n.emojiCategoryPeople));
    await tester.pumpAndSettle();
    await tapEmoji('👋');
    expect(container.read(inputTextProvider), '😀😃👋');
    await tester.pump(const Duration(seconds: 10));
  });
}
