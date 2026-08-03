import 'package:drift/drift.dart';

import 'memory_executor_io.dart'
    if (dart.library.html) 'memory_executor_web.dart' as impl;

/// A drift [QueryExecutor] backed by an in-memory SQLite database, shared by
/// the integration test fixtures across platforms:
///
/// - Desktop/mobile: `NativeDatabase.memory()` (`memory_executor_io.dart`)
/// - Web: in-memory WASM SQLite via `/sqlite3.wasm` (`memory_executor_web.dart`)
QueryExecutor inMemoryExecutor() => impl.inMemoryExecutor();
