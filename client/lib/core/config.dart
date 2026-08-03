import 'dart:convert';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:ourchat/core/const.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'config.freezed.dart';
part 'config.g.dart';

/// Connection info for a single OurChat server.
///
/// [uniqueIdentifier] is the server's self-reported id (from `getServerInfo`).
/// It is `null` until the first successful probe, after which it is the stable
/// key used everywhere a "server id" is needed (DB directory name, SecretStore
/// key namespace, provider family argument). It is more stable than
/// `host:port` because it survives the server moving to a new host.
@freezed
abstract class ServerConfig with _$ServerConfig {
  factory ServerConfig({
    required String host,
    required int port,
    String? uniqueIdentifier,
    String? label,
    bool? isTLS,
  }) = _ServerConfig;
  factory ServerConfig.fromJson(Map<String, dynamic> json) =>
      _$ServerConfigFromJson(json);
}

/// A persisted login identity: "this account on this server". Multiple
/// [SavedAccount]s may share the same [serverId] (several accounts on one
/// server) and the same [accountId] may appear under different servers
/// (numeric ids are only unique per server).
///
/// [accountId] is stored as a plain `int` for JSON-friendliness; convert to
/// `Int64` at the boundary where it meets gRPC.
@freezed
abstract class SavedAccount with _$SavedAccount {
  factory SavedAccount({
    required String serverId,
    required int accountId,
    String? ocid,
    String? email,
    String? avatarKey,
    required DateTime lastLoginAt,
    @Default(true) bool autoLogin,
  }) = _SavedAccount;
  factory SavedAccount.fromJson(Map<String, dynamic> json) =>
      _$SavedAccountFromJson(json);
}

@freezed
abstract class LanguageConfig with _$LanguageConfig {
  const factory LanguageConfig({
    required String languageCode,
    required String scriptCode,
    required String countryCode,
  }) = _LanguageConfig;

  static const defaults = LanguageConfig(
    languageCode: '',
    scriptCode: '',
    countryCode: '',
  );

  factory LanguageConfig.fromJson(Map<String, dynamic> json) =>
      _$LanguageConfigFromJson(json);
}

@freezed
abstract class OurChatConfig with _$OurChatConfig {
  const OurChatConfig._();

  factory OurChatConfig({
    @Default([]) List<ServerConfig> servers,
    @Default([]) List<SavedAccount> savedAccounts,
    String? activeServerId,
    int? activeAccountId,
    @Default(UiDisplayMode.accountSwitcher) UiDisplayMode displayMode,
    @Default(0xFF2196F3) int color,
    @Default('info') String logLevel,
    LanguageConfig? language,
    @Default('https://api.github.com/repos/skyuoi/ourchat/releases')
    String updateSource,
    @JsonKey(includeToJson: false, includeFromJson: false)
    SharedPreferencesWithCache? prefsWithCache,
  }) = _OurChatConfig;

  factory OurChatConfig.fromJson(Map<String, dynamic> json) =>
      _$OurChatConfigFromJson(json);

  static OurChatConfig get defaults {
    return OurChatConfig(
      servers: [ServerConfig(host: 'skyuoi.org', port: 7777)],
    );
  }

  void saveConfig() {
    if (prefsWithCache == null) return;
    prefsWithCache!.setString('config', jsonEncode(toJson()));
  }
}

@Riverpod(keepAlive: true)
class ConfigNotifier extends _$ConfigNotifier {
  @override
  OurChatConfig build() {
    return OurChatConfig.defaults;
  }

  void init(OurChatConfig config) {
    state = config;
  }

  void reload() {
    String? storageConfig;
    if (state.prefsWithCache != null) {
      storageConfig = state.prefsWithCache!.getString('config');
    }
    final loaded = storageConfig != null
        ? OurChatConfig.fromJson(jsonDecode(storageConfig))
        : OurChatConfig.defaults;
    state = state.copyWith(
      servers: loaded.servers,
      savedAccounts: loaded.savedAccounts,
      activeServerId: loaded.activeServerId,
      activeAccountId: loaded.activeAccountId,
      displayMode: loaded.displayMode,
      color: loaded.color,
      logLevel: loaded.logLevel,
      language: loaded.language,
      updateSource: loaded.updateSource,
    );
  }

  void reset() {
    final d = OurChatConfig.defaults;
    state = state.copyWith(
      servers: d.servers,
      savedAccounts: d.savedAccounts,
      activeServerId: d.activeServerId,
      activeAccountId: d.activeAccountId,
      displayMode: d.displayMode,
      color: d.color,
      logLevel: d.logLevel,
      language: d.language,
      updateSource: d.updateSource,
    );
    state.saveConfig();
  }

  void setPrefs(SharedPreferencesWithCache prefs) {
    state = state.copyWith(prefsWithCache: prefs);
  }

  void setLanguage(LanguageConfig language) {
    state = state.copyWith(language: language);
    state.saveConfig();
  }

  void setUpdateSource(String value) {
    state = state.copyWith(updateSource: value);
    state.saveConfig();
  }

  void setColor(int color) {
    state = state.copyWith(color: color);
    state.saveConfig();
  }

  void setLogLevel(String logLevel) {
    state = state.copyWith(logLevel: logLevel);
    state.saveConfig();
  }

  void setDisplayMode(UiDisplayMode mode) {
    state = state.copyWith(displayMode: mode);
    state.saveConfig();
  }

  /// Replace the entire server list (legacy setter, kept for compatibility
  /// with the connect screen).
  void setServers(List<ServerConfig> servers) {
    state = state.copyWith(servers: servers);
    state.saveConfig();
  }

  /// Insert or update a server, keyed by [ServerConfig.uniqueIdentifier] when
  /// available, otherwise by `host:port`.
  void upsertServer(ServerConfig server) {
    final servers = List<ServerConfig>.from(state.servers);
    final i = _indexOfServer(servers, server);
    if (i >= 0) {
      servers[i] = server;
    } else {
      servers.add(server);
    }
    state = state.copyWith(servers: servers);
    state.saveConfig();
  }

  void removeServer(String uniqueIdentifier) {
    state = state.copyWith(
      servers: state.servers
          .where((s) => s.uniqueIdentifier != uniqueIdentifier)
          .toList(),
    );
    state.saveConfig();
  }

  /// Insert or update a saved account, keyed by `(serverId, accountId)`.
  void upsertSavedAccount(SavedAccount account) {
    final accounts = List<SavedAccount>.from(state.savedAccounts);
    final i = accounts.indexWhere(
      (a) => a.serverId == account.serverId && a.accountId == account.accountId,
    );
    if (i >= 0) {
      accounts[i] = account;
    } else {
      accounts.add(account);
    }
    state = state.copyWith(savedAccounts: accounts);
    state.saveConfig();
  }

  void removeSavedAccount(String serverId, int accountId) {
    state = state.copyWith(
      savedAccounts: state.savedAccounts
          .where((a) => !(a.serverId == serverId && a.accountId == accountId))
          .toList(),
    );
    state.saveConfig();
  }

  /// Record which account is currently active. Pass `null` for both to clear.
  void setActiveAccount(String? serverId, int? accountId) {
    state = state.copyWith(
      activeServerId: serverId,
      activeAccountId: accountId,
    );
    state.saveConfig();
  }

  int _indexOfServer(List<ServerConfig> list, ServerConfig s) {
    for (var i = 0; i < list.length; i++) {
      final e = list[i];
      final idMatch =
          s.uniqueIdentifier != null &&
          e.uniqueIdentifier == s.uniqueIdentifier;
      final hpMatch = e.host == s.host && e.port == s.port;
      if (idMatch || hpMatch) return i;
    }
    return -1;
  }
}
