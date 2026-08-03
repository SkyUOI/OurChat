import 'package:drift/drift.dart';
import 'package:drift/wasm.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:sqlite3/wasm.dart' show InMemoryFileSystem, WasmSqlite3;

part 'database_web.g.dart';

const bool _useWorker = true; // Set to false to test without worker

// ── Public (cross-account) schema ─────────────────────────────────────────
// Scoped by `serverId` so the same numeric id on different servers does not
// collide. See database_desktop.dart for the rationale.

class PublicSession extends Table {
  TextColumn get serverId => text()();
  Int64Column get sessionId => int64()();
  TextColumn get name => text()();
  TextColumn get avatarKey => text().nullable()();
  DateTimeColumn get createdTime => dateTime()();
  DateTimeColumn get updatedTime => dateTime()();
  IntColumn get size => integer()();
  TextColumn get description => text()();

  @override
  Set<Column> get primaryKey => {serverId, sessionId};
}

class PublicAccount extends Table {
  TextColumn get serverId => text()();
  Int64Column get id => int64()();
  TextColumn get username => text()();
  TextColumn get status => text().nullable()();
  TextColumn get avatarKey => text().nullable()();
  TextColumn get ocid => text()();
  DateTimeColumn get publicUpdateTime => dateTime()();

  @override
  Set<Column> get primaryKey => {serverId, id};
}

@DriftDatabase(tables: [PublicSession, PublicAccount])
class PublicOurChatDatabase extends _$PublicOurChatDatabase {
  PublicOurChatDatabase([QueryExecutor? executor])
    : super(executor ?? _openPublicConnection());

  @override
  int get schemaVersion => 2;

  @override
  MigrationStrategy get migration => MigrationStrategy(
    onCreate: (m) => m.createAll(),
    onUpgrade: (m, from, to) async {
      for (final t in allTables) {
        await m.deleteTable(t.actualTableName);
      }
      await m.createAll();
    },
  );

  static QueryExecutor _openPublicConnection() {
    if (!_useWorker) {
      return DatabaseConnection.delayed(
        Future<DatabaseConnection>(() async {
          final sqlite3 = await WasmSqlite3.loadFromUrl(
            Uri.parse('/sqlite3.wasm'),
          );
          sqlite3.registerVirtualFileSystem(
            InMemoryFileSystem(),
            makeDefault: true,
          );
          return DatabaseConnection(WasmDatabase.inMemory(sqlite3));
        }),
      );
    }

    return DatabaseConnection.delayed(
      Future<DatabaseConnection>(() async {
        final result = await WasmDatabase.open(
          databaseName: 'publicOurChatDatabase',
          sqlite3Uri: Uri.parse('/sqlite3.wasm'),
          driftWorkerUri: Uri.parse(
            kReleaseMode ? '/drift_worker.min.js' : '/drift_worker.js',
          ),
        );
        return DatabaseConnection(result.resolvedExecutor);
      }),
    );
  }
}

// ── Private (per account-on-a-server) schema ──────────────────────────────

class Account extends Table {
  Int64Column get id => int64()();
  TextColumn get email => text()();
  DateTimeColumn get registerTime => dateTime()();
  DateTimeColumn get updateTime => dateTime()();
  TextColumn get friendsJson => text()();
  TextColumn get sessionsJson => text()();

  DateTimeColumn get latestMsgTime => dateTime()();
}

class Session extends Table {
  Int64Column get sessionId => int64()();
  TextColumn get members => text()();
  TextColumn get roles => text()();
  TextColumn get myPermissions => text()();

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
  OurChatDatabase(String serverId, Int64 id, [QueryExecutor? executor])
    : super(executor ?? _openConnection(serverId, id));

  @override
  int get schemaVersion => 1;

  static QueryExecutor _openConnection(String serverId, Int64 id) {
    if (!_useWorker) {
      return DatabaseConnection.delayed(
        Future<DatabaseConnection>(() async {
          final sqlite3 = await WasmSqlite3.loadFromUrl(
            Uri.parse('/sqlite3.wasm'),
          );
          sqlite3.registerVirtualFileSystem(
            InMemoryFileSystem(),
            makeDefault: true,
          );
          return DatabaseConnection(WasmDatabase.inMemory(sqlite3));
        }),
      );
    }

    return DatabaseConnection.delayed(
      Future<DatabaseConnection>(() async {
        final result = await WasmDatabase.open(
          databaseName: 'OurChatDB_${serverId}_${id.toString()}',
          sqlite3Uri: Uri.parse('/sqlite3.wasm'),
          driftWorkerUri: Uri.parse(
            kReleaseMode ? '/drift_worker.min.js' : '/drift_worker.js',
          ),
        );
        return DatabaseConnection(result.resolvedExecutor);
      }),
    );
  }
}
