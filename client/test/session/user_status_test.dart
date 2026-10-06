import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/friends.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:ourchat/user.dart';
import 'package:ourchat/user_profile_page.dart';
import 'test_harness.dart';

/// A stub account notifier whose `getAccountInfo` never touches the DB or
/// network, so the profile / friends widgets render immediately.
class NoFetchAccount extends StubAccountNotifier {
  NoFetchAccount(super.account);

  @override
  Future<bool> getAccountInfo({bool ignoreCache = false}) async => true;
}

void main() {
  setUpAll(() {
    registerFallbackValue(SetSelfInfoRequest());
  });

  Future<ProviderContainer> pumpSelfInfoDialog(
    WidgetTester tester,
    MockOurChatClient client, {
    AccountData? account,
    List<String> presets = const [],
  }) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ourChatAccountProvider(testServerId, Int64(1)).overrideWith(
          () => NoFetchAccount(account ?? buildTestAccount(Int64(1), 'alice')),
        ),
        presetUserStatusProvider.overrideWithValue(() async => presets),
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

  group('SelfInfoEditDialog user-defined status', () {
    testWidgets('saving carries the entered status as user_defined_status', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      await pumpSelfInfoDialog(tester, client);

      // username / ocid / status fields, in order.
      await tester.enterText(find.byType(TextFormField).at(2), 'on vacation');
      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final captured = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured;
      final request = captured.single as SetSelfInfoRequest;
      expect(request.userDefinedStatus, 'on vacation');
      // The other fields are still sent unchanged.
      expect(request.userName, 'alice');
      expect(request.ocid, '1');
    });

    testWidgets('an emptied status clears user_defined_status', (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      final account = buildTestAccount(
        Int64(1),
        'alice',
      ).copyWith(status: 'busy');
      await pumpSelfInfoDialog(tester, client, account: account);

      await tester.enterText(find.byType(TextFormField).at(2), '');
      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final captured = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured;
      final request = captured.single as SetSelfInfoRequest;
      expect(request.userDefinedStatus, '');
    });

    testWidgets('preset status chips fill the status field', (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.setSelfInfo(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(SetSelfInfoResponse()));

      await pumpSelfInfoDialog(tester, client, presets: const ['Busy', 'Away']);

      expect(find.text(l10n.presetStatus), findsOneWidget);
      await tester.tap(find.text('Away'));
      await tester.pump();

      await tester.tap(find.byIcon(Icons.check));
      await tester.pumpAndSettle();

      final captured = verify(
        () => client.setSelfInfo(captureAny(), options: any(named: 'options')),
      ).captured;
      final request = captured.single as SetSelfInfoRequest;
      expect(request.userDefinedStatus, 'Away');
    });
  });

  group('status display', () {
    testWidgets('friends list shows the friend status as subtitle', (
      tester,
    ) async {
      final client = MockOurChatClient();
      final alice = buildTestAccount(
        Int64(1),
        'alice',
      ).copyWith(friends: [Int64(2), Int64(3)]);
      final bob = buildTestAccount(
        Int64(2),
        'bob',
      ).copyWith(status: 'Listening to music');
      final carol = buildTestAccount(Int64(3), 'carol');

      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
          ourChatAccountProvider(
            testServerId,
            Int64(1),
          ).overrideWith(() => NoFetchAccount(alice)),
          ourChatAccountProvider(
            testServerId,
            Int64(2),
          ).overrideWith(() => NoFetchAccount(bob)),
          ourChatAccountProvider(
            testServerId,
            Int64(3),
          ).overrideWith(() => NoFetchAccount(carol)),
        ],
      );
      addTearDown(container.dispose);
      container.read(thisAccountIdProvider.notifier).setAccountId(Int64(1));

      await tester.pumpWidget(
        buildTestApp(container: container, child: const Friends()),
      );
      await tester.pumpAndSettle();

      expect(find.text('bob'), findsOneWidget);
      expect(find.text('Listening to music'), findsOneWidget);
      // carol has no status: her tile shows only her name, no subtitle text.
      expect(find.text('carol'), findsOneWidget);
      final carolTile = find.ancestor(
        of: find.text('carol'),
        matching: find.byType(ListTile),
      );
      final carolTexts = find.descendant(
        of: carolTile,
        matching: find.byType(Text),
      );
      expect(carolTexts, findsOneWidget); // just the name, no status line
      final bobTile = find.ancestor(
        of: find.text('bob'),
        matching: find.byType(ListTile),
      );
      final bobTexts = find.descendant(
        of: bobTile,
        matching: find.byType(Text),
      );
      expect(bobTexts, findsNWidgets(2)); // name + status subtitle
    });
  });

  testWidgets('user profile page shows a status row', (tester) async {
    final client = MockOurChatClient();
    final bob = buildTestAccount(
      Int64(2),
      'bob',
    ).copyWith(status: 'Listening to music');

    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        ourChatAccountProvider(
          testServerId,
          Int64(2),
        ).overrideWith(() => NoFetchAccount(bob)),
      ],
    );
    addTearDown(container.dispose);
    container.read(thisAccountIdProvider.notifier).setAccountId(Int64(1));

    await tester.pumpWidget(
      buildTestApp(
        container: container,
        child: UserProfilePage(userId: Int64(2)),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text(l10n.status), findsOneWidget);
    expect(find.text('Listening to music'), findsOneWidget);
  });
}
