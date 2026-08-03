import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'test_harness.dart';

void main() {
  setUpAll(() {
    registerFallbackValue(SendMsgRequest());
  });

  testWidgets(
      'sending with a quote target injects quoteMsgId and clears the target',
      (tester) async {
    final client = MockOurChatClient();
    when(() => client.sendMsg(any(), options: any(named: 'options')))
        .thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

    final container = ProviderContainer(overrides: [
      activeAccountTestOverride,
      ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
      overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
    ]);
    addTearDown(container.dispose);

    container.read(quoteTargetProvider.notifier).setQuote(
      UserMsg(senderId: Int64(2), eventId: Int64(5), markdownText: 'quote me'),
    );
    container.read(sessionProvider.notifier).state = SessionState(
      tabIndex: TabType.session,
      currentSessionId: Int64(1),
      currentSessionRecords: const [],
    );

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextFormField), 'reply text');
    await tester.tap(find.text(l10n.send));
    await tester.pumpAndSettle();

    final captured = verify(
      () => client.sendMsg(captureAny(), options: any(named: 'options')),
    ).captured;
    final request = captured.single as SendMsgRequest;
    expect(request.sessionId, Int64(1));
    expect(request.markdownText, 'reply text');
    expect(request.isEncrypted, isFalse);
    expect(request.quoteMsgId, Int64(5));

    expect(container.read(quoteTargetProvider), isNull);
  });

  testWidgets('sending without a quote target sends quoteMsgId zero',
      (tester) async {
    final client = MockOurChatClient();
    when(() => client.sendMsg(any(), options: any(named: 'options')))
        .thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

    final container = ProviderContainer(overrides: [
      activeAccountTestOverride,
      ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
      overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
    ]);
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

    await tester.enterText(find.byType(TextFormField), 'plain reply');
    await tester.tap(find.text(l10n.send));
    await tester.pumpAndSettle();

    final captured = verify(
      () => client.sendMsg(captureAny(), options: any(named: 'options')),
    ).captured;
    final request = captured.single as SendMsgRequest;
    expect(request.quoteMsgId, Int64.ZERO);
  });

  testWidgets('input validator blocks empty sends', (tester) async {
    final client = MockOurChatClient();
    when(() => client.sendMsg(any(), options: any(named: 'options')))
        .thenAnswer((_) => responseFutureOf(SendMsgResponse(msgId: Int64(100))));

    final container = ProviderContainer(overrides: [
      activeAccountTestOverride,
      ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
    ]);
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

    await tester.tap(find.text(l10n.send));
    await tester.pump();
    await tester.pumpAndSettle();

    verifyNever(() => client.sendMsg(any(), options: any(named: 'options')));
    expect(find.text(l10n.cantBeEmpty), findsOneWidget);
  });
}
