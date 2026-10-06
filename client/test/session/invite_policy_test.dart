import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:ourchat/user.dart';
import 'package:ourchat/user_profile_page.dart';
import 'test_harness.dart';

class NoFetchAccount extends StubAccountNotifier {
  NoFetchAccount(super.account);

  @override
  Future<bool> getAccountInfo({bool ignoreCache = false}) async => true;
}

void main() {
  setUpAll(() {
    registerFallbackValue(SetSelfInfoRequest());
  });

  Future<ProviderContainer> pumpDialog(
    WidgetTester tester,
    MockOurChatClient client, {
    AccountData? account,
  }) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ourChatAccountProvider(testServerId, Int64(1)).overrideWith(
          () => NoFetchAccount(account ?? buildTestAccount(Int64(1), 'alice')),
        ),
        presetUserStatusProvider.overrideWithValue(() async => []),
      ],
    );
    addTearDown(container.dispose);
    container.read(thisAccountIdProvider.notifier).setAccountId(Int64(1));

    final accountData = account ?? buildTestAccount(Int64(1), 'alice');
    await tester.pumpWidget(
      buildTestApp(
        container: container,
        child: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog(
              context: context,
              builder: (_) => SelfInfoEditDialog(accountData: accountData),
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

  group('SelfInfoEditDialog session invitation policy (issue #34)', () {
    testWidgets('defaults to "everyone" and saving sends policy 0', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      await pumpDialog(tester, client);

      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final request = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured.single as SetSelfInfoRequest;
      expect(request.sessionInvitationPolicy, 0);
    });

    testWidgets('switching to "friends only" sends policy 1', (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      await pumpDialog(tester, client);

      await tester.tap(find.byType(DropdownButtonFormField<int>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Friends only').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final request = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured.single as SetSelfInfoRequest;
      expect(request.sessionInvitationPolicy, 1);
    });

    testWidgets('a stored "nobody" policy is preselected and saved as 2', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      final account = buildTestAccount(Int64(1), 'alice').copyWith(
        sessionInvitationPolicy: 2,
      );
      await pumpDialog(tester, client, account: account);

      expect(find.text('Nobody'), findsOneWidget);
      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final request = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured.single as SetSelfInfoRequest;
      expect(request.sessionInvitationPolicy, 2);
    });
  });

  group('UserProfilePage policy row (issue #34)', () {
    testWidgets('shows the human-readable policy for others', (tester) async {
      final account = buildTestAccount(Int64(2), 'bob').copyWith(
        isMe: false,
        sessionInvitationPolicy: 1,
      );
      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatAccountProvider(testServerId, Int64(2)).overrideWith(
            () => NoFetchAccount(account),
          ),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: UserProfilePage(userId: Int64(2)),
        ),
      );
      await tester.pump();

      expect(find.text('Who can invite me into sessions'), findsOneWidget);
      expect(find.text('Friends only'), findsOneWidget);
    });

    testWidgets('hides the row when the policy is unknown', (tester) async {
      final account = buildTestAccount(Int64(2), 'bob').copyWith(isMe: false);
      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatAccountProvider(testServerId, Int64(2)).overrideWith(
            () => NoFetchAccount(account),
          ),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        buildTestApp(
          container: container,
          child: UserProfilePage(userId: Int64(2)),
        ),
      );
      await tester.pump();

      expect(find.text('Who can invite me into sessions'), findsNothing);
    });
  });
}
