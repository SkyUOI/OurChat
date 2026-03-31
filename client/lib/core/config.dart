import 'dart:convert';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'config.freezed.dart';
part 'config.g.dart';

@freezed
abstract class ServerConfig with _$ServerConfig {
  factory ServerConfig({required String host, required int port}) =
      _ServerConfig;
  factory ServerConfig.fromJson(Map<String, dynamic> json) =>
      _$ServerConfigFromJson(json);
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
    @Default(0xFF2196F3) int color,
    @Default('info') String logLevel,
    @Default('') String recentAccount,
    @Default('') String recentPassword,
    @Default('') String recentAvatarUrl,
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
    // Config is loaded eagerly in main() and injected via init()
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
      color: loaded.color,
      logLevel: loaded.logLevel,
      recentAccount: loaded.recentAccount,
      recentPassword: loaded.recentPassword,
      recentAvatarUrl: loaded.recentAvatarUrl,
      language: loaded.language,
      updateSource: loaded.updateSource,
    );
  }

  void reset() {
    final d = OurChatConfig.defaults;
    state = state.copyWith(
      servers: d.servers,
      color: d.color,
      logLevel: d.logLevel,
      recentAccount: d.recentAccount,
      recentPassword: d.recentPassword,
      recentAvatarUrl: d.recentAvatarUrl,
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

  void setRecent(String recentAccount, String recentPassword) {
    state = state.copyWith(
      recentAccount: recentAccount,
      recentPassword: recentPassword,
    );
    state.saveConfig();
  }

  void setAvatarUrl(String key) {
    state = state.copyWith(recentAvatarUrl: key);
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

  void setServers(List<ServerConfig> servers) {
    state = state.copyWith(servers: servers);
    state.saveConfig();
  }
}
