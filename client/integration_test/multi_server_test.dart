import 'dart:convert';

import 'package:drift/native.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/auth_notifier.dart';
import 'package:ourchat/main.dart';

import 'helpers/multi_server_fixture.dart';

/// Exercises the multi-server / multi-account features added in phase 2 of
/// the client refactor, against TWO live servers (ports 7777 + 7778):
///
/// 1. Concurrent logins produce one [OurChatInstance] per (server, account),
///    with distinct serverIds and distinct private databases.
/// 2. `activeAccount` switching updates the legacy globals consistently.
/// 3. Event systems are isolated: a message on server A lands only in A's
///    private DB and is delivered only to A's listeners.
/// 4. Two accounts on the SAME server coexist as separate instances.
///
/// Skips when either server is unreachable.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final fx = MultiServerFixture();

  tearDownAll(() async => await fx.tearDown());

  testWidgets(
    'two servers, one account each: distinct instances + active switching',
    (_) async {
      final ok = await fx.setUp();
      if (!ok) {
        markTestSkipped('servers at localhost:7777 / 7778 are not reachable');
        return;
      }

      // ── ① Two concurrent instances, one per server ──
      final instances = fx.container.read(instancesProvider);
      expect(instances.length, 2);
      final keys = instances.keys.toList();
      expect(
        keys[0].serverId,
        isNot(keys[1].serverId),
        reason: 'each account belongs to its own server',
      );
      expect(
        {keys[0].serverId, keys[1].serverId},
        {fx.serverIdA, fx.serverIdB},
      );
      expect(
        instances[keys[0]]!.privateDB,
        isNot(same(instances[keys[1]]!.privateDB)),
        reason: 'each instance keeps its own private database',
      );

      // ── ② Active starts at A ──
      expect(
        fx.container.read(activeAccountProvider),
        AccountKey(fx.serverIdA, fx.accountIdA),
      );
      expect(fx.container.read(activeServerIdProvider), fx.serverIdA);
      expect(fx.container.read(activeAccountIdProvider), fx.accountIdA);

      // ── ③ Switch active to B; legacy globals follow ──
      fx.switchTo(fx.serverIdB, fx.accountIdB);
      expect(fx.container.read(activeServerIdProvider), fx.serverIdB);
      expect(fx.container.read(activeAccountIdProvider), fx.accountIdB);
      expect(fx.container.read(thisAccountIdProvider), fx.accountIdB);
      expect(
        privateDB,
        same(instances[AccountKey(fx.serverIdB, fx.accountIdB)]!.privateDB),
      );

      // ── ④ Switch back to A ──
      fx.switchTo(fx.serverIdA, fx.accountIdA);
      expect(fx.container.read(activeServerIdProvider), fx.serverIdA);
    },
  );

  testWidgets(
    'event stream isolation: message on server A stays out of server B',
    (_) async {
      final ok = await fx.setUp();
      if (!ok) {
        markTestSkipped('servers at localhost:7777 / 7778 are not reachable');
        return;
      }

      // Provision a peer + session on server A only.
      final bob = await fx.appA.registerUser();
      final sessionA = await fx.userA
          .createSession([bob])
          .then((r) => r.sessionId);

      // Send a message as account A on server A.
      await fx.sendAsAccountA(
        sessionId: sessionA,
        markdownText: 'hello from server A',
      );

      // Account A's event system delivers it.
      final received = await fx.waitForMsgA(
        (m) => m.markdownText == 'hello from server A',
      );
      expect(received.eventId, isNotNull);

      // Give B's stream a beat; it must NOT have seen it.
      await Future.delayed(const Duration(milliseconds: 800));
      final aRecords = await fx.dbRecordsFor(fx.serverIdA, fx.accountIdA);
      final bRecords = await fx.dbRecordsFor(fx.serverIdB, fx.accountIdB);

      expect(
        aRecords.any(
          (r) =>
              (r.sessionId?.toString() ?? '') == sessionA.toString() &&
              ((jsonDecode(r.data) as Map<String, dynamic>)['markdown_text'] ==
                  'hello from server A'),
        ),
        isTrue,
        reason: 'server A message persisted into A private DB',
      );
      expect(
        bRecords,
        isEmpty,
        reason: 'server B private DB must not contain server A messages',
      );

      // Cleanup
      try {
        await fx.userA.deleteSession(sessionA);
      } catch (_) {}
    },
  );

  testWidgets('two accounts on the SAME server coexist as separate instances', (
    _,
  ) async {
    final ok = await fx.setUp();
    if (!ok) {
      markTestSkipped('servers at localhost:7777 / 7778 are not reachable');
      return;
    }

    // Register + login a second account on server A.
    final userA2 = await fx.appA.registerUser();
    final okLogin = await fx.container
        .read(authProvider.notifier)
        .login(
          email: userA2.email,
          password: userA2.password,
          server: fx.serverA,
        );
    expect(okLogin, isTrue);
    final accountA2 = fx.container.read(authProvider).accountId!;

    final dbA2 = database.OurChatDatabase(
      fx.serverIdA,
      accountA2,
      NativeDatabase.memory(),
    );
    fx.container
        .read(instancesProvider.notifier)
        .add(
          OurChatInstance(
            serverId: fx.serverIdA,
            accountId: accountA2,
            server: fx.serverA,
            privateDB: dbA2,
          ),
        );

    final instances = fx.container.read(instancesProvider);
    expect(instances.length, 3);
    final onServerA = instances.values.where((i) => i.serverId == fx.serverIdA);
    expect(
      onServerA.length,
      2,
      reason: 'two accounts share server A as separate instances',
    );
    expect(onServerA.map((i) => i.accountId).toSet(), {
      fx.accountIdA,
      accountA2,
    });
    expect(
      onServerA.first.privateDB,
      isNot(same(onServerA.last.privateDB)),
      reason: 'per-account private DB even on the same server',
    );
  });
}
