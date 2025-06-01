import 'package:drift/drift.dart';
import 'package:drift_flutter/drift_flutter.dart';
import 'package:fixnum/fixnum.dart';
import 'package:path_provider/path_provider.dart';

part 'ourchat_database.g.dart';

class PublicAccount extends Table {
  Int64Column get id => int64()();
  TextColumn get username => text()();
  TextColumn get status => text()();
  TextColumn get avatarKey => text()();
  TextColumn get ocid => text()();
  DateTimeColumn get publicUpdateTime => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}

@DriftDatabase(tables: [PublicAccount])
class PublicOurchatDatabase extends _$PublicOurchatDatabase {
  PublicOurchatDatabase([QueryExecutor? executor])
      : super(executor ?? _openConnection());

  @override
  int get schemaVersion => 1;

  static QueryExecutor _openConnection() {
    return driftDatabase(
      name: 'publicOurchatDB',
      native: const DriftNativeOptions(
        databaseDirectory: getApplicationSupportDirectory,
      ),
    );
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

  @override
  Set<Column> get primaryKey => {id};
}

@DriftDatabase(tables: [Account])
class OurchatDatabase extends _$OurchatDatabase {
  OurchatDatabase(id, [QueryExecutor? executor])
      : super(executor ?? _openConnection(id));

  @override
  int get schemaVersion => 1;

  static QueryExecutor _openConnection(Int64 id) {
    return driftDatabase(
      name: 'OurchatDB_${id.toString()}',
      native: const DriftNativeOptions(
        databaseDirectory: getApplicationSupportDirectory,
      ),
    );
  }
}
