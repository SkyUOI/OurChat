import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/join_session_dialog.dart';
import 'package:ourchat/service/ourchat/session/join_session/v1/join_session.pb.dart';
import 'test_harness.dart';

/// Like [StubAccountNotifier] but with a no-op `getAccountInfo` so the join
/// flow never touches the database or the network.
class JoinStubAccount extends StubAccountNotifier {
  JoinStubAccount(super.account);

  @override
  Future<bool> getAccountInfo({bool ignoreCache = false}) async => true;
}

void main() {
  setUpAll(() {
    registerFallbackValue(JoinSessionRequest());
  });

  Future<ProviderContainer> pumpDialog(
    WidgetTester tester,
    MockOurChatClient client,
  ) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ourChatAccountProvider(testServerId, Int64(1)).overrideWith(
          () => JoinStubAccount(buildTestAccount(Int64(1), 'alice')),
        ),
      ],
    );
    addTearDown(container.dispose);
    container.read(thisAccountIdProvider.notifier).setAccountId(Int64(1));

    await tester.pumpWidget(
      buildTestApp(
        container: container,
        child: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog(
              context: context,
              builder: (_) => JoinSessionDialog(sessionId: Int64(42)),
            ),
            child: const Text('open-dialog'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open-dialog'));
    await tester.pumpAndSettle();
    return container;
  }

  testWidgets(
    'non-member session join dialog submits joinSession with the session id',
    (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.joinSession(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(JoinSessionResponse()));

      await pumpDialog(tester, client);

      expect(find.text(l10n.joinSession), findsOneWidget);
      expect(find.text('42'), findsOneWidget);

      await tester.enterText(find.byType(TextFormField), 'let me in');
      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final captured = verify(
        () => client.joinSession(captureAny(), options: any(named: 'options')),
      ).captured;
      final request = captured.single as JoinSessionRequest;
      expect(request.sessionId, Int64(42));
      expect(request.leaveMessage, 'let me in');
      // The dialog closes after a successful request.
      expect(find.text(l10n.joinSession), findsNothing);
    },
  );

  testWidgets('the leave message may stay empty', (tester) async {
    final client = MockOurChatClient();
    when(
      () => client.joinSession(any(), options: any(named: 'options')),
    ).thenAnswer((_) => responseFutureOf(JoinSessionResponse()));

    await pumpDialog(tester, client);

    await tester.tap(find.byIcon(Icons.check));
    await tester.pumpAndSettle();

    final captured = verify(
      () => client.joinSession(captureAny(), options: any(named: 'options')),
    ).captured;
    final request = captured.single as JoinSessionRequest;
    expect(request.sessionId, Int64(42));
    expect(request.leaveMessage, isEmpty);
  });

  testWidgets('a rejected join request keeps the dialog open', (tester) async {
    final client = MockOurChatClient();
    when(
      () => client.joinSession(any(), options: any(named: 'options')),
    ).thenThrow(grpc.GrpcError.notFound('not found'));

    await pumpDialog(tester, client);

    await tester.enterText(find.byType(TextFormField), 'hi');
    await tester.tap(find.byIcon(Icons.check));
    await tester.pumpAndSettle();

    verify(
      () => client.joinSession(any(), options: any(named: 'options')),
    ).called(1);
    expect(find.text(l10n.joinSession), findsOneWidget);
  });
}
