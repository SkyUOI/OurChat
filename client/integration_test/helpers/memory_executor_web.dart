import 'package:drift/drift.dart';
import 'package:drift/wasm.dart';
import 'package:sqlite3/wasm.dart' show InMemoryFileSystem, WasmSqlite3;

/// In-memory WASM SQLite for the web platform, mirroring the non-worker path
/// in `lib/core/database_web.dart`. Requires `/sqlite3.wasm` to be served.
QueryExecutor inMemoryExecutor() {
  return DatabaseConnection.delayed(
    Future<DatabaseConnection>(() async {
      final sqlite3 = await WasmSqlite3.loadFromUrl(Uri.parse('/sqlite3.wasm'));
      sqlite3.registerVirtualFileSystem(
        InMemoryFileSystem(),
        makeDefault: true,
      );
      return DatabaseConnection(WasmDatabase.inMemory(sqlite3));
    }),
  );
}
