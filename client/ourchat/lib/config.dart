import 'package:localstorage/localstorage.dart';

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

  void checkConfig() {
    var defaultConfig = getDefaultConfig();
    for (var key in defaultConfig.keys) {
      if (!data!.containsKey(key)) {
        data![key] = defaultConfig[key];
      }
    }
  }

  void loadConfig() {
    var defaultConfig = getDefaultConfig();
    data = {};
    for (var key in defaultConfig.keys) {
      var value = localStorage.getItem(key);
      if (value != null) {
        data![key] = value;
      }
    }
    checkConfig();
  }

  void saveConfig() {
    checkConfig();
    for (var key in data!.keys) {
      localStorage.setItem(key, data![key]);
    }
  }
}
