import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:ourchat/core/log.dart';

/// Manage config entries of application
///
class OurchatConfig {
  late Map<String, dynamic> data;
  SharedPreferencesWithCache? prefsWithCache;

  Map<String, dynamic> getDefaultConfig() {
    // 默认配置
    return {
      "servers": [
        {"host": "localhost", "port": 7777},
      ],
      "color": 0xFF2196F3,
      "log_level": defaultLogLevel,
      "recent_account": "",
      "recent_password": ""
    };
  }

  void reset() {
    data = getDefaultConfig();
    saveConfig();
  }

  void checkConfig() {
    logger.i("check config");
    var defaultConfig = getDefaultConfig();
    for (var key in defaultConfig.keys) {
      if (!data.containsKey(key)) {
        data[key] = defaultConfig[key];
        logger.t("$key does not exist,use default value: $defaultConfig[key]");
      }
    }
    logger.i("check config done");
  }

  void reload() {
    logger.i("load config");
    var defaultConfig = getDefaultConfig();
    String? storageConfig;
    if (prefsWithCache != null) {
      storageConfig = prefsWithCache!.getString("config");
    }
    if (storageConfig == null) {
      data = defaultConfig;
    } else {
      data = jsonDecode(storageConfig);
    }
    checkConfig();
    logger.i("load config done");
    logger.i("config: $data");
  }

  OurchatConfig() {
    SharedPreferencesWithCache.create(
            cacheOptions: const SharedPreferencesWithCacheOptions())
        .then((prefs) {
      prefsWithCache = prefs;
      reload();
    });
    data = getDefaultConfig();
    reload();
  }

  void saveConfig() {
    logger.i("save config");
    // checkConfig();
    if (prefsWithCache == null) {
      logger.w("prefsWithCache is null,return");
      return;
    }
    prefsWithCache!.setString("config", jsonEncode(data));
    logger.i("save config done");
  }

  operator [](key) => data[key];

  operator []=(key, value) {
    data[key] = value;
    saveConfig();
  }
}
