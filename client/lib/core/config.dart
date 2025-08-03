import 'package:localstorage/localstorage.dart';
import 'dart:convert';

import 'package:ourchat/core/log.dart';

/// Manage config entries of application
///
/// # Warning
/// Please call [initLocalStorage] first
class OurchatConfig {
  late Map<String, dynamic> data;

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
    var storageConfig = localStorage.getItem("config");
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
    reload();
  }

  void saveConfig() {
    logger.i("save config");
    // checkConfig();
    localStorage.setItem("config", jsonEncode(data));
    logger.i("save config done");
  }

  operator [](key) => data[key];

  operator []=(key, value) {
    data[key] = value;
    saveConfig();
  }
}
