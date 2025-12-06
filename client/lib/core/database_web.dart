import 'package:drift/drift.dart';
import 'package:drift/wasm.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:sqlite3/wasm.dart' show InMemoryFileSystem, WasmSqlite3;

part 'database_web.g.dart';

const bool _useWorker = true; // Set to false to test without worker

class PublicSession extends Table {
  Int64Column get sessionId => int64()();
  TextColumn get name => text()();
  TextColumn get avatarKey => text().nullable()();
  DateTimeColumn get createdTime => dateTime()();
  DateTimeColumn get updatedTime => dateTime()();
  IntColumn get size => integer()();
  TextColumn get description => text()();

  @override
  Set<Column> get primaryKey => {sessionId};
}

class PublicAccount extends Table {
  Int64Column get id => int64()();
  TextColumn get username => text()();
  TextColumn get status => text().nullable()();
  TextColumn get avatarKey => text().nullable()();
  TextColumn get ocid => text()();
  DateTimeColumn get publicUpdateTime => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}

@DriftDatabase(tables: [PublicSession, PublicAccount])
class PublicOurChatDatabase extends _$PublicOurChatDatabase {
  PublicOurChatDatabase([QueryExecutor? executor])
      : super(executor ?? _openConnection());

  @override
  int get schemaVersion => 1;

  static QueryExecutor _openConnection() {
    if (!_useWorker) {
      // Test without worker using in-memory database
      return DatabaseConnection.delayed(Future<DatabaseConnection>(() async {
        final sqlite3 =
            await WasmSqlite3.loadFromUrl(Uri.parse('/sqlite3.wasm'));
        sqlite3.registerVirtualFileSystem(InMemoryFileSystem(),
            makeDefault: true);
        return DatabaseConnection(WasmDatabase.inMemory(sqlite3));
      }));
    }

    // Use the stable WasmDatabase.open API for web platform
    return DatabaseConnection.delayed(Future<DatabaseConnection>(() async {
      final result = await WasmDatabase.open(
        databaseName: 'publicOurChatDatabase',
        sqlite3Uri: Uri.parse('/sqlite3.wasm'),
        driftWorkerUri: Uri.parse(
          kReleaseMode ? '/drift_worker.min.js' : '/drift_worker.js',
        ),
      );
      return DatabaseConnection(result.resolvedExecutor);
    }));
  }
}

class Account extends Table {
  Int64Column get id => int64()();
  TextColumn get email => text()();
  DateTimeColumn get registerTime => dateTime()();
  DateTimeColumn get updateTime => dateTime()();
  TextColumn get friendsJson => text()();
  TextColumn get sessionsJson => text()();

  // 客户端独有字段
  DateTimeColumn get latestMsgTime => dateTime()();
}

class Session extends Table {
  Int64Column get sessionId => int64()();
  TextColumn get members => text()();
  TextColumn get roles => text()();

  @override
  Set<Column> get primaryKey => {sessionId};
}

class Record extends Table {
  Int64Column get eventId => int64()();
  Int64Column get sessionId => int64().nullable()();
  IntColumn get eventType => integer()();
  Int64Column get sender => int64()();
  DateTimeColumn get time => dateTime()();
  TextColumn get data => text()();
  IntColumn get read => integer().withDefault(const Constant(0))();

  @override
  Set<Column> get primaryKey => {eventId};
}

@DriftDatabase(tables: [Account, Session, Record])
class OurChatDatabase extends _$OurChatDatabase {
  OurChatDatabase(Int64 id, [QueryExecutor? executor])
      : super(executor ?? _openConnection(id));

  @override
  int get schemaVersion => 1;

  static QueryExecutor _openConnection(Int64 id) {
    if (!_useWorker) {
      // Test without worker using in-memory database
      return DatabaseConnection.delayed(Future<DatabaseConnection>(() async {
        final sqlite3 =
            await WasmSqlite3.loadFromUrl(Uri.parse('/sqlite3.wasm'));
        sqlite3.registerVirtualFileSystem(InMemoryFileSystem(),
            makeDefault: true);
        return DatabaseConnection(WasmDatabase.inMemory(sqlite3));
      }));
    }

    // Use the stable WasmDatabase.open API for web platform
    return DatabaseConnection.delayed(Future<DatabaseConnection>(() async {
      final result = await WasmDatabase.open(
        databaseName: 'OurChatDB_${id.toString()}',
        sqlite3Uri: Uri.parse('/sqlite3.wasm'),
        driftWorkerUri: Uri.parse(
          kReleaseMode ? '/drift_worker.min.js' : '/drift_worker.js',
        ),
      );
      return DatabaseConnection(result.resolvedExecutor);
    }));
  }
}
