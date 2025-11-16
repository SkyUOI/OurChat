// Desktop/Mobile logger implementation
import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';
import 'dart:io';

// Desktop version - file + console
Future<Logger> createDesktopLogger(Level logLevel) async {
  var path = await getApplicationDocumentsDirectory();
  var file = File('${path.path}/ourchat.log');
  file.openWrite(mode: FileMode.writeOnlyAppend);

  return Logger(
      output: MultiOutput([FileOutput(file: file), ConsoleOutput()]),
      level: logLevel,
      printer: PrettyPrinter(
        dateTimeFormat: (time) {
          return "${time.year}-${time.month}-${time.day} ${time.hour}:${time.minute}:${time.second}.${time.millisecond}";
        },
      ));
}

// Desktop stub for web function - not used on desktop
Logger createWebLogger(Level logLevel) {
  // This should never be called on desktop platform
  throw UnsupportedError(
      'createWebLogger is not supported on desktop platform');
}
