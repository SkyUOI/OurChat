import 'package:drift/drift.dart';
import 'package:drift/native.dart';

QueryExecutor inMemoryExecutor() => NativeDatabase.memory();
