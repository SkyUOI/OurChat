import 'package:cached_network_image/cached_network_image.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/session_record.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'test_harness.dart';

void main() {
  setUpAll(() {
    registerFallbackValue(SendMsgRequest());
  });

  group('MessageWidget http image fallback (#221)', () {
    Future<void> pumpMessage(WidgetTester tester, String markdown) async {
      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(
            FakeOurChatServer(MockOurChatClient()),
          ),
        ],
      );
      addTearDown(container.dispose);
      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: MessageWidget(
            msg: UserMsg(markdownText: markdown),
            opacity: 1.0,
          ),
        ),
      );
      await tester.pump();
    }

    testWidgets(
      'a plain http image renders via CachedNetworkImage with the untrusted badge',
      (tester) async {
        await pumpMessage(tester, '![cat](https://example.com/cat.png)');

        // The image renders through CachedNetworkImage (still loading /
        // placeholder in the test environment), not the error text.
        expect(
          find.byWidgetPredicate(
            (w) =>
                w is CachedNetworkImage &&
                w.imageUrl == 'https://example.com/cat.png',
          ),
          findsOneWidget,
        );
        expect(find.text(l10n.internalError), findsNothing);
        // The untrusted-source warning badge.
        expect(find.byIcon(Icons.warning), findsOneWidget);
        expect(find.byTooltip(l10n.untrustedImageSource), findsOneWidget);
      },
    );

    testWidgets(
      'an in:// external image renders via CachedNetworkImage with the untrusted badge',
      (tester) async {
        await pumpMessage(tester, '![cat](in://https,example.com/cat.png)');

        // The avatar also renders as a CachedNetworkImage, so find the image
        // widget by its decoded URL.
        final imageFinder = find.byWidgetPredicate(
          (w) =>
              w is CachedNetworkImage &&
              w.imageUrl == 'https://example.com/cat.png',
        );
        expect(imageFinder, findsOneWidget);
        expect(find.byIcon(Icons.warning), findsOneWidget);
        expect(find.byTooltip(l10n.untrustedImageSource), findsOneWidget);
      },
    );
  });

  group('SessionTab external image send flow (#221)', () {
    Future<ProviderContainer> pumpSessionTab(
      WidgetTester tester,
      MockOurChatClient client,
    ) async {
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
      return container;
    }

    testWidgets(
      'choosing "keep external links" sends the image as in:// markdown',
      (tester) async {
        final client = MockOurChatClient();
        when(
          () => client.sendMsg(any(), options: any(named: 'options')),
        ).thenAnswer(
          (_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))),
        );

        await pumpSessionTab(tester, client);
        await tester.enterText(
          find.byType(TextFormField),
          'look: ![cat](https://example.com/cat.png)',
        );
        await tester.tap(find.text(l10n.send));
        await tester.pump();

        // The choice dialog appears before anything is sent.
        expect(find.text(l10n.externalImagesTitle), findsOneWidget);
        verifyNever(
          () => client.sendMsg(any(), options: any(named: 'options')),
        );

        await tester.tap(find.text(l10n.keepExternalLinks));
        await tester.pumpAndSettle();

        final captured = verify(
          () => client.sendMsg(captureAny(), options: any(named: 'options')),
        ).captured;
        final request = captured.single as SendMsgRequest;
        expect(
          request.markdownText,
          'look: ![cat](in://https,example.com/cat.png)',
        );
      },
    );

    testWidgets('cancelling the dialog aborts the send', (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.sendMsg(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

      await pumpSessionTab(tester, client);
      await tester.enterText(
        find.byType(TextFormField),
        '![cat](https://example.com/cat.png)',
      );
      await tester.tap(find.text(l10n.send));
      await tester.pump();

      await tester.tap(find.text(l10n.cancel));
      await tester.pumpAndSettle();

      verifyNever(() => client.sendMsg(any(), options: any(named: 'options')));
      // The draft is preserved so nothing is lost.
      expect(containerInputText(tester), '![cat](https://example.com/cat.png)');
    });

    testWidgets('a message without external images sends directly', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.sendMsg(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

      await pumpSessionTab(tester, client);
      await tester.enterText(
        find.byType(TextFormField),
        '![local](io://0) plain text',
      );
      await tester.tap(find.text(l10n.send));
      await tester.pumpAndSettle();

      expect(find.text(l10n.externalImagesTitle), findsNothing);
      final captured = verify(
        () => client.sendMsg(captureAny(), options: any(named: 'options')),
      ).captured;
      final request = captured.single as SendMsgRequest;
      expect(request.markdownText, '![local](io://0) plain text');
    });
  });
}

/// The draft text currently held by [inputTextProvider] in the test container.
///
/// The container is created inside `pumpSessionTab`, so recover it from the
/// element tree via the still-mounted [SessionTab]'s ProviderScope.
String containerInputText(WidgetTester tester) {
  final element = tester.element(find.byType(SessionTab));
  final container = ProviderScope.containerOf(element);
  return container.read(inputTextProvider);
}
