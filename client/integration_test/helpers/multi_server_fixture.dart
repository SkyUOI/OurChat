import 'dart:async';
import 'dart:ui' show Locale;

import 'package:flutter/foundation.dart' show debugPrint;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/test/test_flutter_secure_storage_platform.dart';
import 'package:flutter_secure_storage_platform_interface/flutter_secure_storage_platform_interface.dart';
import 'package:ourchat/core/auth_notifier.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/e2ee.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/secret_store.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';

import 'memory_executor.dart';
import 'oc_test_app.dart';
import 'oc_test_user.dart';

/// Multi-server harness: logs into ONE account on EACH of two live servers,
/// builds two runtime [OurChatInstance]s in a single container, starts both
/// event systems, and exposes helpers to exercise/verify isolation between
/// them. Skips when either server is unreachable.
class MultiServerFixture {
  MultiServerFixture({this.portA = 7777, this.portB = 7778});

  final int portA;
  final int portB;

  late OcTestApp appA;
  late OcTestApp appB;
  late OcTestUser userA;
  late OcTestUser userB;
  late ProviderContainer container;

  late OurChatServer serverA;
  late OurChatServer serverB;
  late String serverIdA;
  late String serverIdB;
  late Int64 accountIdA;
  late Int64 accountIdB;

  bool _ready = false;
  bool _setUpRan = false;
  bool get isReady => _ready;

  Future<bool> setUp() async {
    appA = OcTestApp('localhost', portA, false);
    appB = OcTestApp('localhost', portB, false);
    _setUpRan = true;
    final probeA = await appA.probe();
    final probeB = await appB.probe();
    debugPrint('[MultiServer] probe A=$probeA B=$probeB');
    if (!probeA || !probeB) return false;

    // Register one account per server (provisioning only).
    userA = await appA.registerUser();
    userB = await appB.registerUser();

    // In-memory SecretStore.
    FlutterSecureStoragePlatform.instance = TestFlutterSecureStoragePlatform(
      {},
    );
    l10n = await AppLocalizations.delegate.load(const Locale('en'));
    publicDB = database.PublicOurChatDatabase(inMemoryExecutor());

    // Probe getServerInfo to populate each server's uniqueIdentifier.
    serverA = OurChatServer('localhost', portA, false);
    final infoA = await serverA.getServerInfo();
    serverB = OurChatServer('localhost', portB, false);
    final infoB = await serverB.getServerInfo();
    debugPrint('[MultiServer] getServerInfo A=$infoA B=$infoB');
    if (infoA != okStatusCode || infoB != okStatusCode) return false;
    serverIdA = serverA.uniqueIdentifier!;
    serverIdB = serverB.uniqueIdentifier!;
    debugPrint('[MultiServer] serverIdA=$serverIdA serverIdB=$serverIdB');
    if (serverIdA == serverIdB) {
      // Two distinct servers must report distinct unique identifiers.
      throw StateError('servers share the same uniqueIdentifier: $serverIdA');
    }

    await SecretStore.savePrivateKey(
      serverIdA,
      userA.id,
      userA.keyPair.privateKey,
    );
    await SecretStore.savePrivateKey(
      serverIdB,
      userB.id,
      userB.keyPair.privateKey,
    );

    container = ProviderContainer(
      overrides: [ourChatServerProvider.overrideWithValue(serverA)],
    );

    // Login account A (real AuthNotifier.login against server A).
    final okA = await container
        .read(authProvider.notifier)
        .login(email: userA.email, password: userA.password, server: serverA);
    debugPrint(
      '[MultiServer] login A ok=$okA err=${container.read(authProvider).error}',
    );
    if (!okA) return false;
    accountIdA = container.read(authProvider).accountId!;

    // Login account B (real AuthNotifier.login against server B).
    final okB = await container
        .read(authProvider.notifier)
        .login(email: userB.email, password: userB.password, server: serverB);
    debugPrint(
      '[MultiServer] login B ok=$okB err=${container.read(authProvider).error}',
    );
    if (!okB) return false;
    accountIdB = container.read(authProvider).accountId!;

    // Build the two runtime instances.
    final dbA = database.OurChatDatabase(
      serverIdA,
      accountIdA,
      inMemoryExecutor(),
    );
    container
        .read(instancesProvider.notifier)
        .add(
          OurChatInstance(
            serverId: serverIdA,
            accountId: accountIdA,
            server: serverA,
            privateDB: dbA,
          ),
        );
    final dbB = database.OurChatDatabase(
      serverIdB,
      accountIdB,
      inMemoryExecutor(),
    );
    container
        .read(instancesProvider.notifier)
        .add(
          OurChatInstance(
            serverId: serverIdB,
            accountId: accountIdB,
            server: serverB,
            privateDB: dbB,
          ),
        );

    // Active account = A; mirror into legacy globals.
    container
        .read(activeAccountProvider.notifier)
        .set(AccountKey(serverIdA, accountIdA));
    privateDB = dbA;
    container.read(thisAccountIdProvider.notifier).setAccountId(accountIdA);

    // Start both event systems (real streams).
    container
        .read(ourChatEventSystemProvider(serverIdA, accountIdA).notifier)
        .listenEvents();
    container
        .read(ourChatEventSystemProvider(serverIdB, accountIdB).notifier)
        .listenEvents();

    await Future.delayed(const Duration(milliseconds: 500));

    _ready = true;
    return true;
  }

  // ── send / receive helpers per account ──

  /// Switch the active account, mirroring what `switchActive` does for a real
  /// widget tree: updates the active pointer + legacy globals.
  void switchTo(String serverId, Int64 accountId) {
    final key = AccountKey(serverId, accountId);
    container.read(activeAccountProvider.notifier).set(key);
    final inst = container.read(instancesProvider)[key];
    if (inst != null) {
      privateDB = inst.privateDB;
      container
          .read(thisAccountIdProvider.notifier)
          .setAccountId(inst.accountId);
    }
  }

  Future<SendMsgResponse?> sendAsAccountA({
    required Int64 sessionId,
    required String markdownText,
  }) {
    return UserMsg(sessionId: sessionId, markdownText: markdownText).send(
      serverA,
      container.read(e2eeStoreProvider(serverIdA, accountIdA).notifier),
      sessionId,
    );
  }

  Future<SendMsgResponse?> sendAsAccountB({
    required Int64 sessionId,
    required String markdownText,
  }) {
    return UserMsg(sessionId: sessionId, markdownText: markdownText).send(
      serverB,
      container.read(e2eeStoreProvider(serverIdB, accountIdB).notifier),
      sessionId,
    );
  }

  Future<UserMsg> waitForMsgA(
    bool Function(UserMsg) matcher, {
    Duration timeout = const Duration(seconds: 15),
  }) {
    return _waitForMsg(serverIdA, accountIdA, matcher, timeout);
  }

  Future<UserMsg> waitForMsgB(
    bool Function(UserMsg) matcher, {
    Duration timeout = const Duration(seconds: 15),
  }) {
    return _waitForMsg(serverIdB, accountIdB, matcher, timeout);
  }

  Future<UserMsg> _waitForMsg(
    String serverId,
    Int64 accountId,
    bool Function(UserMsg) matcher,
    Duration timeout,
  ) async {
    final eventSystem = container.read(
      ourChatEventSystemProvider(serverId, accountId).notifier,
    );
    final completer = Completer<UserMsg>();
    void cb(dynamic eventObj) {
      if (eventObj is UserMsg && !completer.isCompleted && matcher(eventObj)) {
        completer.complete(eventObj);
      }
    }

    eventSystem.addListener(FetchMsgsResponse_RespondEventType.msg, cb);
    try {
      return await completer.future.timeout(timeout);
    } finally {
      eventSystem.removeListener(FetchMsgsResponse_RespondEventType.msg, cb);
    }
  }

  /// The private DB rows recorded for [serverId]/[accountId].
  Future<List<database.RecordData>> dbRecordsFor(
    String serverId,
    Int64 accountId,
  ) {
    final key = AccountKey(serverId, accountId);
    final inst = container.read(instancesProvider)[key];
    final db = inst?.privateDB;
    if (db == null) return Future.value(const []);
    return db.select(db.record).get();
  }

  Future<void> tearDown() async {
    if (!_setUpRan) return;
    if (_ready) {
      container
          .read(ourChatEventSystemProvider(serverIdA, accountIdA).notifier)
          .stopListening();
      container
          .read(ourChatEventSystemProvider(serverIdB, accountIdB).notifier)
          .stopListening();
      container.dispose();
    }
    await appA.dispose();
    await appB.dispose();
  }
}
