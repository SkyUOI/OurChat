import 'package:flutter/widgets.dart';
import 'package:localstorage/localstorage.dart';
import 'dart:convert';

import 'package:ourchat/log.dart';

/// Manage config entries of application
///
/// # Warning
/// Please call [initLocalStorage] first
class OurchatConfig {
  late Map<String, dynamic> data;

  Map<String, dynamic> getDefaultConfig() {
    return {
      "servers": [
        {"host": "localhost", "port": 7777},
      ],
      "color": 0xFF2196F3,
      "log_level": defaultLogLevel
    };
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
    checkConfig();
    localStorage.setItem("config", jsonEncode(data));
    logger.i("save config done");
  }
}

late OurchatConfig _ourchatConfigInternal;
OurchatConfigState ourchatConfig = OurchatConfigState();

void initConfig() {
  _ourchatConfigInternal = OurchatConfig();
}

class OurchatConfigState extends ChangeNotifier {
  OurchatConfig get data => _ourchatConfigInternal;

  operator [](String key) => _ourchatConfigInternal.data[key];

  operator []=(String key, dynamic value) {
    _ourchatConfigInternal.data[key] = value;
    _ourchatConfigInternal.saveConfig();
    notifyListeners();
  }

  void set(Map<String, dynamic> value) {
    _ourchatConfigInternal.data = value;
    _ourchatConfigInternal.saveConfig();
    notifyListeners();
  }
}
