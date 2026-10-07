import 'dart:convert';

import 'package:drift/drift.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart'
    show
        instancesProvider,
        activeAccountProvider,
        ourChatServerProvider,
        publicDB;

import 'helpers/memory_executor.dart';

/// Regression test for the web int64 JSON precision bug (found by manually
/// testing the Flutter web build): event `data` maps used to store Int64
/// values as JSON numbers. On the web build (dart2js) those numbers are JS
/// doubles, which lose precision above 2^53 — a stored id decoded back as
/// 2107740679317487600 no longer matched the real 2107740679317487616, which
/// broke the friend-request accept/refuse buttons among other things.
/// Int64 values are now stored as exact decimal strings.
///
/// This test round-trips values above 2^53 through a real drift database and
/// the real [UserMsg.loadFromDB] extraction. On the Dart VM it passes
/// trivially (64-bit ints are exact); it is meant to run on Chrome
/// (`--device chrome`) where the JS number path is exercised.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final pdb = database.OurChatDatabase(
    'test-server',
    Int64(1),
    inMemoryExecutor(),
  );

  // The quote field extraction in loadFromDB reads the sender through the
  // account provider, which needs a registered instance for its private DB,
  // and a server to talk to. Register a minimal instance pointed at a port
  // that refuses connections so the fetch fails fast (the extraction is
  // fault-tolerant) instead of stalling on DNS.
  final server = OurChatServer('127.0.0.1', 1, false);
  final container = ProviderContainer(
    overrides: [ourChatServerProvider.overrideWithValue(server)],
  );
  final ref = container.read(refProvider);
  // getAccountInfo caches account data in the shared public DB (a global in
  // main.dart) — point it at an in-memory database for the test.
  publicDB = database.PublicOurChatDatabase(inMemoryExecutor());
  final instance = OurChatInstance(
    serverId: 'test-server',
    accountId: Int64(2),
    server: server,
    privateDB: pdb,
  );
  container.read(instancesProvider.notifier).add(instance);
  container.read(activeAccountProvider.notifier).set(instance.key);

  testWidgets('UserMsg quote int64 fields round-trip through drift + JSON', (
    _,
  ) async {
    // 2^53 + 1: the smallest integer a JS double cannot represent.
    const quoteMsgId = '9007199254740993';
    // A real snowflake-style user id observed in production.
    const quoteSenderId = '2107740679317487616';

    final msg = UserMsg(
      eventId: Int64(1),
      senderId: Int64(2),
      sendTime: OurChatTime.fromDatetime(DateTime.now()),
      markdownText: 'round trip',
      quoteMsgId: Int64.parseInt(quoteMsgId),
      quoteSenderId: Int64.parseInt(quoteSenderId),
      quoteMarkdownText: 'quoted',
    );

    await msg.saveToDB(pdb);

    final row = await (pdb.select(
      pdb.record,
    )..where((u) => u.eventId.equals(BigInt.from(1)))).getSingle();

    // The stored JSON must carry the exact decimal strings (a JSON number
    // would round-trip as 9007199254740992 on the web).
    final stored = jsonDecode(row.data) as Map<String, dynamic>;
    expect(stored['quote_msg_id'], quoteMsgId);
    expect(stored['quote_sender_id'], quoteSenderId);

    // And loadFromDB must restore the exact Int64 values.
    final loaded = UserMsg();
    await loaded.loadFromDB(ref, 'test-server', pdb, row);
    expect(loaded.quoteMsgId, Int64.parseInt(quoteMsgId));
    expect(loaded.quoteSenderId, Int64.parseInt(quoteSenderId));
  });
}

/// Exposes a [Ref] from the container for code under test that takes one.
final refProvider = Provider<Ref>((ref) => ref);
