import 'package:drift/native.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/home.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/server_setting.dart';

import 'test_harness.dart';

/// Verifies that once the user is logged in they can still reach the server
/// management page to add another server's account (multi-server support).
void main() {
  testWidgets(
    'AccountSwitcher offers "add server" and navigates to ServerSetting',
    (tester) async {
      final fakeServer = FakeOurChatServer(MockOurChatClient());
      final container = ProviderContainer(
        overrides: [
          activeAccountTestOverride,
          ourChatServerProvider.overrideWithValue(fakeServer),
          overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
        ],
      );
      addTearDown(container.dispose);

      // Register a live instance so the switcher shows the active account.
      final instance = OurChatInstance(
        serverId: testServerId,
        accountId: Int64(1),
        server: fakeServer,
        privateDB: database.OurChatDatabase(
          testServerId,
          Int64(1),
          NativeDatabase.memory(),
        ),
      );
      container.read(instancesProvider.notifier).add(instance);

      await tester.pumpWidget(
        buildTestApp(container: container, child: const AccountSwitcher()),
      );
      await tester.pump();

      // The switcher shows the active account.
      expect(find.textContaining('alice'), findsOneWidget);

      // Open the menu and tap "add server".
      await tester.tap(find.textContaining('alice'));
      await tester.pumpAndSettle();
      expect(find.text(l10n.addServer), findsOneWidget);

      await tester.tap(find.text(l10n.addServer));
      await tester.pumpAndSettle();

      // It navigates to the server management page.
      expect(find.byType(ServerSetting), findsOneWidget);
    },
  );
}
