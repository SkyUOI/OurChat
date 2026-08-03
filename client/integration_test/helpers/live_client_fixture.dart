import 'dart:async';
import 'dart:ui' show Locale;

import 'package:drift/native.dart';
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

import 'oc_test_app.dart';
import 'oc_test_user.dart';

/// Exercises REAL client app logic against a live server:
///
/// - Registers a user via raw gRPC (setup only)
/// - Logs in via the REAL [AuthNotifier.login] (exercises auth code path)
/// - Starts the REAL [OurChatEventSystem.listenEvents] (exercises stream
///   parsing, [quoteFieldsFromMsg], drift persistence, listener dispatch)
/// - [sendAsClient] calls the REAL [UserMsg.send] (exercises send path +
///   E2EE decision)
///
/// Only [ourChatServerProvider] is overridden (to point at the test server).
/// Everything else — [e2eeStoreProvider], [ourChatAccountProvider],
/// [authProvider], [ourChatEventSystemProvider] — runs unmodified production
/// code. Drift DBs are in-memory ([NativeDatabase.memory]); [SecretStore] is
/// backed by [TestFlutterSecureStoragePlatform].
class LiveClientFixture {
  LiveClientFixture();

  late final OcTestApp app;
  late final OcTestUser user;
  late final ProviderContainer container;
  late final Int64 accountId;
  late final String serverId;

  bool _ready = false;
  bool _setUpRan = false;
  bool get isReady => _ready;

  /// Probe + register + login + start event system. Returns false (no throw)
  /// when the server is unreachable.
  Future<bool> setUp({String host = 'localhost', int port = 7777}) async {
    app = OcTestApp(host, port, false);
    _setUpRan = true;
    if (!await app.probe()) return false;

    // Register a user via raw gRPC (provisioning only).
    user = await app.registerUser();

    // In-memory SecretStore backed by the official test platform.
    FlutterSecureStoragePlatform.instance = TestFlutterSecureStoragePlatform(
      {},
    );

    // Initialize l10n without a widget tree (UserMsg.send error path needs it).
    l10n = await AppLocalizations.delegate.load(const Locale('en'));

    // Real server + container. Probe getServerInfo first so the server's
    // uniqueIdentifier is populated (needed as serverId everywhere).
    final server = OurChatServer(host, port, false);
    final infoCode = await server.getServerInfo();
    if (infoCode != okStatusCode) return false;
    serverId = server.uniqueIdentifier!;

    // Pre-seed the private key so E2EE loadKey works for later E2EE tests.
    await SecretStore.savePrivateKey(
      serverId,
      user.id,
      user.keyPair.privateKey,
    );

    // In-memory public database.
    publicDB = database.PublicOurChatDatabase(NativeDatabase.memory());

    container = ProviderContainer(
      overrides: [ourChatServerProvider.overrideWithValue(server)],
    );

    // REAL login — exercises AuthNotifier.login (auth RPC, token injection,
    // thisAccountId update, private-key load).
    final ok = await container
        .read(authProvider.notifier)
        .login(email: user.email, password: user.password, server: server);
    if (!ok) return false;

    accountId = container.read(authProvider).accountId!;

    // Build the runtime instance for this login and register it as active.
    final instancePrivateDB = database.OurChatDatabase(
      serverId,
      accountId,
      NativeDatabase.memory(),
    );
    final instance = OurChatInstance(
      serverId: serverId,
      accountId: accountId,
      server: server,
      privateDB: instancePrivateDB,
    );
    container.read(instancesProvider.notifier).add(instance);
    container.read(activeAccountProvider.notifier).set(instance.key);
    privateDB = instancePrivateDB;

    // REAL event system — exercises listenEvents (stream connect, parsing,
    // dedup, drift write, listener dispatch).
    container
        .read(ourChatEventSystemProvider(serverId, accountId).notifier)
        .listenEvents();

    // Give the stream a moment to establish before tests send messages.
    await Future.delayed(const Duration(milliseconds: 500));

    _ready = true;
    return true;
  }

  // ── real client send path ──

  /// Send via the REAL [UserMsg.send] — exercises E2EE decision,
  /// [safeRequest], and request packaging.
  Future<SendMsgResponse?> sendAsClient({
    required Int64 sessionId,
    required String markdownText,
    Int64? quoteMsgId,
    List<String> involvedFiles = const [],
  }) {
    return UserMsg(
      sessionId: sessionId,
      markdownText: markdownText,
      quoteMsgId: quoteMsgId,
      involvedFiles: involvedFiles,
    ).send(
      container.read(ourChatServerProvider),
      container.read(e2eeStoreProvider(serverId, accountId).notifier),
      sessionId,
    );
  }

  // ── real client receive path ──

  /// Wait for a msg event that satisfies [matcher], delivered through the REAL
  /// [OurChatEventSystem] listener pipeline (after quoteFieldsFromMsg +
  /// saveToDB).
  Future<UserMsg> waitForMsg(
    bool Function(UserMsg) matcher, {
    Duration timeout = const Duration(seconds: 15),
  }) async {
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

  /// Read all Record rows from the real in-memory drift DB.
  Future<List<database.RecordData>> dbRecords() {
    return privateDB!.select(privateDB!.record).get();
  }

  Future<void> tearDown() async {
    if (!_setUpRan) return;
    if (_ready) {
      container
          .read(ourChatEventSystemProvider(serverId, accountId).notifier)
          .stopListening();
      container.dispose();
    }
    await app.dispose();
  }
}
