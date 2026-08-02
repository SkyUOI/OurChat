import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/session_record.dart';
import 'package:ourchat/session/state.dart';
import 'test_harness.dart';

void main() {
  late ProviderContainer container;

  ProviderContainer makeContainer() {
    final c = ProviderContainer(overrides: [
      ourChatServerProvider.overrideWithValue(
        FakeOurChatServer(MockOurChatClient()),
      ),
      overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
      overrideAccount(Int64(2), buildTestAccount(Int64(2), 'bob', displayName: 'Bob')),
    ]);
    addTearDown(c.dispose);
    return c;
  }

  Future<void> pumpMessage(WidgetTester tester, UserMsg msg) async {
    container = makeContainer();
    await tester.pumpWidget(
      buildTestApp(
        container: container,
        child: MessageWidget(msg: msg, opacity: 1.0),
      ),
    );
    await tester.pump();
  }

  group('MessageWidget quote block', () {
    testWidgets('shows quoted sender name and preview text', (tester) async {
      await pumpMessage(
        tester,
        UserMsg(
          senderId: Int64(1),
          eventId: Int64(10),
          markdownText: 'hello',
          quoteMsgId: Int64(5),
          quoteSenderId: Int64(2),
          quoteMarkdownText: 'quoted text',
        ),
      );

      expect(find.text('Bob'), findsOneWidget);
      expect(find.text('quoted text'), findsOneWidget);
      expect(find.text('alice'), findsOneWidget);
    });

    testWidgets('shows [image] placeholder for an image quote', (tester) async {
      await pumpMessage(
        tester,
        UserMsg(
          senderId: Int64(1),
          eventId: Int64(10),
          markdownText: 'hello',
          quoteMsgId: Int64(5),
          quoteSenderId: Int64(2),
          quoteMarkdownText: '![pic](io://0)',
          quoteInvolvedFiles: const ['file_key'],
        ),
      );

      expect(find.text('[${l10n.image}]'), findsOneWidget);
    });

    testWidgets('shows unavailable placeholder when quoted text is empty',
        (tester) async {
      await pumpMessage(
        tester,
        UserMsg(
          senderId: Int64(1),
          eventId: Int64(10),
          markdownText: 'hello',
          quoteMsgId: Int64(5),
          quoteSenderId: Int64(2),
        ),
      );

      expect(find.text(l10n.quoteUnavailable), findsOneWidget);
    });

    testWidgets('no quote block rendered when there is no quote',
        (tester) async {
      await pumpMessage(
        tester,
        UserMsg(senderId: Int64(1), eventId: Int64(10), markdownText: 'hi'),
      );

      expect(find.text(l10n.quoteUnavailable), findsNothing);
    });
  });

  group('MessageWidget long-press quote menu', () {
    testWidgets('long-press offers Quote and sets the quote target',
        (tester) async {
      final msg = UserMsg(
        senderId: Int64(1),
        eventId: Int64(10),
        markdownText: 'hello',
      );
      container = makeContainer();
      // Keep the auto-dispose quoteTargetProvider alive so its state can be
      // observed after the menu selection.
      final sub = container.listen(quoteTargetProvider, (_, _) {});
      addTearDown(sub.close);
      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: MessageWidget(msg: msg, opacity: 1.0),
        ),
      );

      await tester.longPress(find.byType(MessageWidget));
      await tester.pumpAndSettle();
      expect(find.text(l10n.quote), findsOneWidget);

      await tester.tap(find.text(l10n.quote));
      await tester.pumpAndSettle();
      expect(container.read(quoteTargetProvider)?.eventId, Int64(10));
      expect(container.read(quoteTargetProvider)?.markdownText, 'hello');
    });

    testWidgets('dismissing the menu does not set a quote target',
        (tester) async {
      final msg = UserMsg(
        senderId: Int64(1),
        eventId: Int64(10),
        markdownText: 'hello',
      );
      container = makeContainer();
      // Keep the auto-dispose quoteTargetProvider alive so its state can be
      // observed after the menu selection.
      final sub = container.listen(quoteTargetProvider, (_, _) {});
      addTearDown(sub.close);
      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: MessageWidget(msg: msg, opacity: 1.0),
        ),
      );

      await tester.longPress(find.byType(MessageWidget));
      await tester.pumpAndSettle();
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();
      expect(container.read(quoteTargetProvider), isNull);
    });
  });
}
