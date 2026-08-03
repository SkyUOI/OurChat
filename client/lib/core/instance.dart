import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/server.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'instance.g.dart';

/// Composite identity of a logged-in account: which server + which account id.
///
/// Used as a `Map` key. Numeric account ids are only unique *per server*, so
/// the pair is required to identify an identity globally.
class AccountKey {
  final String serverId;
  final Int64 accountId;

  const AccountKey(this.serverId, this.accountId);

  @override
  bool operator ==(Object other) =>
      other is AccountKey &&
      other.serverId == serverId &&
      other.accountId == accountId;

  @override
  int get hashCode => Object.hash(serverId, accountId);

  @override
  String toString() => 'AccountKey($serverId, $accountId)';
}

/// A running logged-in identity: the live gRPC connection to one server plus
/// that account's private local database. One of these exists per concurrent
/// login. Multiple instances (different servers, or different accounts on the
/// same server) coexist in [Instances].
class OurChatInstance {
  final String serverId;
  final Int64 accountId;
  final OurChatServer server;
  final database.OurChatDatabase privateDB;

  OurChatInstance({
    required this.serverId,
    required this.accountId,
    required this.server,
    required this.privateDB,
  });

  AccountKey get key => AccountKey(serverId, accountId);
}

/// Registry of all currently-live instances (concurrent logins). Keyed by
/// [AccountKey]. The "active" one (the one the UI is currently focused on) is
/// tracked separately by [ActiveAccount].
@Riverpod(keepAlive: true)
class Instances extends _$Instances {
  @override
  Map<AccountKey, OurChatInstance> build() => {};

  void add(OurChatInstance instance) {
    final next = Map<AccountKey, OurChatInstance>.from(state);
    next[instance.key] = instance;
    state = next;
  }

  void remove(AccountKey key) {
    final next = Map<AccountKey, OurChatInstance>.from(state)..remove(key);
    state = next;
  }

  OurChatInstance? get(AccountKey key) => state[key];

  /// The instance for the currently active account, if any.
  OurChatInstance? get active {
    final key = ref.read(activeAccountProvider);
    if (key == null) return null;
    return state[key];
  }
}

/// The account the UI is currently focused on. Phase 1 keeps a single active
/// account (mirroring the legacy single-server behaviour); phase 2 will let
/// the user switch between concurrent accounts.
@Riverpod(keepAlive: true)
class ActiveAccount extends _$ActiveAccount {
  @override
  AccountKey? build() => null;

  void set(AccountKey? key) {
    state = key;
  }

  void clear() {
    state = null;
  }
}

/// The server id of the active account, if any. Derived from
/// [ActiveAccount]; rebuilds when the active account changes.
@Riverpod(keepAlive: true)
class ActiveServerId extends _$ActiveServerId {
  @override
  String? build() {
    return ref.watch(activeAccountProvider)?.serverId;
  }
}

/// The active account id, if any. Derived from [ActiveAccount].
@Riverpod(keepAlive: true)
class ActiveAccountId extends _$ActiveAccountId {
  @override
  Int64? build() {
    return ref.watch(activeAccountProvider)?.accountId;
  }
}

/// Find a live instance for [serverId], preferring the active one when it is
/// on that server. Returns null if no account on that server is logged in.
OurChatInstance? instanceForServer(
  ProviderContainer container,
  String serverId,
) {
  final instances = container.read(instancesProvider);
  final active = container.read(activeAccountProvider);
  if (active != null) {
    final a = instances[active];
    if (a != null && a.serverId == serverId) return a;
  }
  for (final i in instances.values) {
    if (i.serverId == serverId) return i;
  }
  return null;
}

/// The private DB to use for data belonging to [serverId] (the active account
/// on that server, or the first logged-in account there).
database.OurChatDatabase? privateDBFor(
  ProviderContainer container,
  String serverId,
) {
  return instanceForServer(container, serverId)?.privateDB;
}
