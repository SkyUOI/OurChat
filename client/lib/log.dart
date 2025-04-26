import 'dart:io';

import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';

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
  var path = await getApplicationDocumentsDirectory();
  var file = File('${path.path}/ourchat.log');
  file.openWrite(mode: FileMode.writeOnlyAppend);
  logger = Logger(
      output: MultiOutput([FileOutput(file: file), ConsoleOutput()]),
      level: logLevel);
  logger.i("Logger has been initialized successfully, File ${path.path}");
}

const logLevels = ["debug", "info", "warning", "error", "trace", "fatal"];
const defaultLogLevel = "info";
