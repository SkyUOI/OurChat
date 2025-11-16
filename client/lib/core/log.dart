import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:logger/logger.dart';

// Conditionally import platform-specific logger implementations
import 'package:ourchat/core/log_desktop.dart'
    if (dart.library.io) 'package:ourchat/core/log_web.dart';

Logger logger = Logger();

Level convertStrIntoLevel(String level) {
  switch (level) {
    case "debug":
      return Level.debug;
    case "info":
      return Level.info;
    case "warning":
      return Level.warning;
    case "error":
      return Level.error;
    case "trace":
      return Level.trace;
    case "fatal":
      return Level.fatal;
    default:
      return Level.info;
  }
}

Future<void> constructLogger(Level logLevel) async {
  if (kIsWeb) {
    // Web platform: Use console output only
    logger = createWebLogger(logLevel);
    logger.i("Web Logger has been initialized successfully (console only)");
  } else {
    // Desktop/Mobile platforms: Use file + console output
    logger = await createDesktopLogger(logLevel);
    logger.i("Logger has been initialized successfully");
  }
}

const logLevels = ["debug", "info", "warning", "error", "trace", "fatal"];
const defaultLogLevel = "info";
