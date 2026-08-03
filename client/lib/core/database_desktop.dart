import 'package:drift/drift.dart';
import 'package:drift_flutter/drift_flutter.dart';
import 'package:fixnum/fixnum.dart';
import 'package:path_provider/path_provider.dart';
import 'dart:io';

part 'database_desktop.g.dart';

// ── Public (cross-account) schema ─────────────────────────────────────────
// Tables here are scoped by `serverId` (= ServerConfig.uniqueIdentifier):
// numeric ids are only unique per server, so the same account/session id on
// two different servers must not collide.

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
      // v2 scopes public tables by serverId — incompatible with v1. With no
      // shipped users we simply rebuild the schema from scratch.
      for (final t in allTables) {
        await m.deleteTable(t.actualTableName);
      }
      await m.createAll();
    },
  );

  static QueryExecutor _openPublicConnection() {
    return driftDatabase(
      name: 'publicOurChatDatabase',
      native: const DriftNativeOptions(
        databaseDirectory: getApplicationSupportDirectory,
      ),
    );
  }
}

// ── Private (per account-on-a-server) schema ──────────────────────────────
// Each (serverId, accountId) gets its own SQLite file under
// `<appSupport>/servers/<serverId>/`, so no serverId column is needed here.

class Account extends Table {
  Int64Column get id => int64()();
  TextColumn get email => text()();
  DateTimeColumn get registerTime => dateTime()();
  DateTimeColumn get updateTime => dateTime()();
  TextColumn get friendsJson => text()();
  TextColumn get sessionsJson => text()();

  // Client-only field
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
    return driftDatabase(
      name: 'OurChatDB_${serverId}_${id.toString()}',
      native: DriftNativeOptions(
        databaseDirectory: () async {
          final base = await getApplicationSupportDirectory();
          final dir = Directory('${base.path}/servers/$serverId');
          if (!await dir.exists()) {
            await dir.create(recursive: true);
          }
          return dir;
        },
      ),
    );
  }
}
