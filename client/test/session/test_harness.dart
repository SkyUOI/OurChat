import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

/// Build an [AccountData] with sensible defaults so tests don't have to spell
/// out every required field.
AccountData buildTestAccount(
  Int64 id,
  String username, {
  String? displayName,
}) {
  final now = DateTime.now();
  return AccountData(
    id: id,
    username: username,
    ocid: '$id',
    displayName: displayName,
    isMe: false,
    publicUpdateTime: OurChatTime.fromDatetime(now),
    updatedTime: OurChatTime.fromDatetime(now),
    registerTime: OurChatTime.fromDatetime(now),
    lastCheckTime: now,
    friends: const [],
    sessions: const [],
  );
}

/// A mocktail mock of the generated gRPC client. Un-stubbed RPCs will throw;
/// tests must stub the specific calls they exercise.
class MockOurChatClient extends Mock implements OurChatServiceClient {}

/// The server id used by widget tests when overriding parameterized providers.
const String testServerId = 'test-server';

/// Override that makes the widget under test see a logged-in active account on
/// [testServerId], so UI reads of `activeServerIdProvider` /
/// `activeAccountIdProvider` return the expected values.
final activeAccountTestOverride = activeAccountProvider.overrideWithValue(
  AccountKey(testServerId, Int64(1)),
);

/// A notifier that returns a canned [AccountData], so widget tests don't need
/// the database or network. Must be a real notifier subclass (not
/// `overrideWithValue`) because the UI reads `.notifier` for `avatarUrl()`.
class StubAccountNotifier extends OurChatAccount {
  StubAccountNotifier(this.account);

  final AccountData account;

  @override
  AccountData build(String serverId, Int64 id) => account;
}

/// Override [ourChatAccountProvider] for [id] (on the test server) to return
/// [account].
Override overrideAccount(Int64 id, AccountData account) {
  return ourChatAccountProvider(testServerId, id).overrideWith(
    () => StubAccountNotifier(account),
  );
}

/// An [OurChatServer] whose `newStub()` returns a test-controlled client,
/// so widget tests never touch the network.
class FakeOurChatServer extends OurChatServer {
  FakeOurChatServer(this.client) : super('localhost', 7777, false);

  final OurChatServiceClient client;

  @override
  OurChatServiceClient newStub() => client;
}

/// A [ClientCall] that hands out a single pre-built response without any
/// transport. Used to build a resolvable `ResponseFuture` for mock stubs.
class _ValueClientCall<R> extends ClientCall<dynamic, R> {
  _ValueClientCall(this._value)
      : super(
          ClientMethod<dynamic, R>(
            '/fake',
            (request) => const <int>[],
            (bytes) => throw UnimplementedError(),
          ),
          const Stream.empty(),
          CallOptions(),
        );

  final R _value;

  @override
  Stream<R> get response => Stream.value(_value);

  @override
  Future<Map<String, String>> get headers => Future.value(const {});

  @override
  Future<Map<String, String>> get trailers => Future.value(const {});
}

/// A gRPC `ResponseFuture` that resolves to [value] — useful for stubbing
/// unary RPCs on a [MockOurChatClient].
ResponseFuture<T> responseFutureOf<T>(T value) {
  return ResponseFuture<T>(_ValueClientCall<T>(value));
}

/// Wrap a widget in a `ProviderScope` + `MaterialApp` with OurChat
/// localizations. Setting `l10n` happens in a `LayoutBuilder` (the same way
/// `MainApp` does), before `child` builds.
Widget buildTestApp({
  required ProviderContainer container,
  required Widget child,
}) {
  return UncontrolledProviderScope(
    container: container,
    child: MaterialApp(
      scaffoldMessengerKey: rootScaffoldMessengerKey,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: LayoutBuilder(
          builder: (context, constraints) {
            l10n = AppLocalizations.of(context)!;
            return child;
          },
        ),
      ),
    ),
  );
}
