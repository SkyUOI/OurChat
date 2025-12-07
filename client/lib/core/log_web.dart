// Web-specific logger implementation
import 'package:logger/logger.dart';

// Web version - console only
Logger createWebLogger(Level logLevel) {
  return Logger(
      output: ConsoleOutput(),
      level: logLevel,
      printer: PrettyPrinter(
        dateTimeFormat: (time) {
          return "${time.year}-${time.month}-${time.day} ${time.hour}:${time.minute}:${time.second}.${time.millisecond}";
        },
      ));
}

// Web stub for desktop function - not used on web
Future<Logger> createDesktopLogger(Level logLevel) async {
  // This should never be called on web platform
  throw UnsupportedError(
      'createDesktopLogger is not supported on web platform');
}
