import 'package:localstorage/localstorage.dart';
import 'package:logger/logger.dart';

class OurchatConfig {
  Map<String, dynamic>? data;

  Map<String, dynamic> getDefaultConfig() {
    return {
      "server_address": "127.0.0.1",
      "ws_port": "7777",
      "http_port": "7778",
      "reconnection_attempt": "5",
      "reconnection_interval": "5",
      "language": "en-us",
    };
  }

  Logger logger = Logger();

  void checkConfig() {
    logger.i("check config");
    var defaultConfig = getDefaultConfig();
    for (var key in defaultConfig.keys) {
      if (!data!.containsKey(key)) {
        data![key] = defaultConfig[key];
        logger.t("$key does not exist,use default value: $defaultConfig[key]");
      }
    }
    logger.i("check config done");
  }

  void loadConfig() {
    logger.i("load config");
    var defaultConfig = getDefaultConfig();
    data = {};
    for (var key in defaultConfig.keys) {
      var value = localStorage.getItem(key);
      if (value != null) {
        data![key] = value;
      }
    }
    checkConfig();
    logger.i("load config done");
  }

  void saveConfig() {
    logger.i("save config");
    checkConfig();
    for (var key in data!.keys) {
      localStorage.setItem(key, data![key]);
    }
    logger.i("save config done");
  }
}
