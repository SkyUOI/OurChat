// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'database.dart';

// ignore_for_file: type=lint
class $PublicSessionTable extends PublicSession
    with TableInfo<$PublicSessionTable, PublicSessionData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $PublicSessionTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _sessionIdMeta =
      const VerificationMeta('sessionId');
  @override
  late final GeneratedColumn<BigInt> sessionId = GeneratedColumn<BigInt>(
      'session_id', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: false);
  static const VerificationMeta _nameMeta = const VerificationMeta('name');
  @override
  late final GeneratedColumn<String> name = GeneratedColumn<String>(
      'name', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _avatarKeyMeta =
      const VerificationMeta('avatarKey');
  @override
  late final GeneratedColumn<String> avatarKey = GeneratedColumn<String>(
      'avatar_key', aliasedName, true,
      type: DriftSqlType.string, requiredDuringInsert: false);
  static const VerificationMeta _createdTimeMeta =
      const VerificationMeta('createdTime');
  @override
  late final GeneratedColumn<int> createdTime = GeneratedColumn<int>(
      'created_time', aliasedName, false,
      type: DriftSqlType.int, requiredDuringInsert: true);
  static const VerificationMeta _updatedTimeMeta =
      const VerificationMeta('updatedTime');
  @override
  late final GeneratedColumn<int> updatedTime = GeneratedColumn<int>(
      'updated_time', aliasedName, false,
      type: DriftSqlType.int, requiredDuringInsert: true);
  static const VerificationMeta _sizeMeta = const VerificationMeta('size');
  @override
  late final GeneratedColumn<int> size = GeneratedColumn<int>(
      'size', aliasedName, false,
      type: DriftSqlType.int, requiredDuringInsert: true);
  static const VerificationMeta _descriptionMeta =
      const VerificationMeta('description');
  @override
  late final GeneratedColumn<String> description = GeneratedColumn<String>(
      'description', aliasedName, true,
      type: DriftSqlType.string, requiredDuringInsert: false);
  @override
  List<GeneratedColumn> get $columns =>
      [sessionId, name, avatarKey, createdTime, updatedTime, size, description];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'public_session';
  @override
  VerificationContext validateIntegrity(Insertable<PublicSessionData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('session_id')) {
      context.handle(_sessionIdMeta,
          sessionId.isAcceptableOrUnknown(data['session_id']!, _sessionIdMeta));
    }
    if (data.containsKey('name')) {
      context.handle(
          _nameMeta, name.isAcceptableOrUnknown(data['name']!, _nameMeta));
    } else if (isInserting) {
      context.missing(_nameMeta);
    }
    if (data.containsKey('avatar_key')) {
      context.handle(_avatarKeyMeta,
          avatarKey.isAcceptableOrUnknown(data['avatar_key']!, _avatarKeyMeta));
    }
    if (data.containsKey('created_time')) {
      context.handle(
          _createdTimeMeta,
          createdTime.isAcceptableOrUnknown(
              data['created_time']!, _createdTimeMeta));
    } else if (isInserting) {
      context.missing(_createdTimeMeta);
    }
    if (data.containsKey('updated_time')) {
      context.handle(
          _updatedTimeMeta,
          updatedTime.isAcceptableOrUnknown(
              data['updated_time']!, _updatedTimeMeta));
    } else if (isInserting) {
      context.missing(_updatedTimeMeta);
    }
    if (data.containsKey('size')) {
      context.handle(
          _sizeMeta, size.isAcceptableOrUnknown(data['size']!, _sizeMeta));
    } else if (isInserting) {
      context.missing(_sizeMeta);
    }
    if (data.containsKey('description')) {
      context.handle(
          _descriptionMeta,
          description.isAcceptableOrUnknown(
              data['description']!, _descriptionMeta));
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {sessionId};
  @override
  PublicSessionData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return PublicSessionData(
      sessionId: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}session_id'])!,
      name: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}name'])!,
      avatarKey: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}avatar_key']),
      createdTime: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}created_time'])!,
      updatedTime: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}updated_time'])!,
      size: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}size'])!,
      description: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}description']),
    );
  }

  @override
  $PublicSessionTable createAlias(String alias) {
    return $PublicSessionTable(attachedDatabase, alias);
  }
}

class PublicSessionData extends DataClass
    implements Insertable<PublicSessionData> {
  final BigInt sessionId;
  final String name;
  final String? avatarKey;
  final int createdTime;
  final int updatedTime;
  final int size;
  final String? description;
  const PublicSessionData(
      {required this.sessionId,
      required this.name,
      this.avatarKey,
      required this.createdTime,
      required this.updatedTime,
      required this.size,
      this.description});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['session_id'] = Variable<BigInt>(sessionId);
    map['name'] = Variable<String>(name);
    if (!nullToAbsent || avatarKey != null) {
      map['avatar_key'] = Variable<String>(avatarKey);
    }
    map['created_time'] = Variable<int>(createdTime);
    map['updated_time'] = Variable<int>(updatedTime);
    map['size'] = Variable<int>(size);
    if (!nullToAbsent || description != null) {
      map['description'] = Variable<String>(description);
    }
    return map;
  }

  PublicSessionCompanion toCompanion(bool nullToAbsent) {
    return PublicSessionCompanion(
      sessionId: Value(sessionId),
      name: Value(name),
      avatarKey: avatarKey == null && nullToAbsent
          ? const Value.absent()
          : Value(avatarKey),
      createdTime: Value(createdTime),
      updatedTime: Value(updatedTime),
      size: Value(size),
      description: description == null && nullToAbsent
          ? const Value.absent()
          : Value(description),
    );
  }

  factory PublicSessionData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return PublicSessionData(
      sessionId: serializer.fromJson<BigInt>(json['sessionId']),
      name: serializer.fromJson<String>(json['name']),
      avatarKey: serializer.fromJson<String?>(json['avatarKey']),
      createdTime: serializer.fromJson<int>(json['createdTime']),
      updatedTime: serializer.fromJson<int>(json['updatedTime']),
      size: serializer.fromJson<int>(json['size']),
      description: serializer.fromJson<String?>(json['description']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'sessionId': serializer.toJson<BigInt>(sessionId),
      'name': serializer.toJson<String>(name),
      'avatarKey': serializer.toJson<String?>(avatarKey),
      'createdTime': serializer.toJson<int>(createdTime),
      'updatedTime': serializer.toJson<int>(updatedTime),
      'size': serializer.toJson<int>(size),
      'description': serializer.toJson<String?>(description),
    };
  }

  PublicSessionData copyWith(
          {BigInt? sessionId,
          String? name,
          Value<String?> avatarKey = const Value.absent(),
          int? createdTime,
          int? updatedTime,
          int? size,
          Value<String?> description = const Value.absent()}) =>
      PublicSessionData(
        sessionId: sessionId ?? this.sessionId,
        name: name ?? this.name,
        avatarKey: avatarKey.present ? avatarKey.value : this.avatarKey,
        createdTime: createdTime ?? this.createdTime,
        updatedTime: updatedTime ?? this.updatedTime,
        size: size ?? this.size,
        description: description.present ? description.value : this.description,
      );
  PublicSessionData copyWithCompanion(PublicSessionCompanion data) {
    return PublicSessionData(
      sessionId: data.sessionId.present ? data.sessionId.value : this.sessionId,
      name: data.name.present ? data.name.value : this.name,
      avatarKey: data.avatarKey.present ? data.avatarKey.value : this.avatarKey,
      createdTime:
          data.createdTime.present ? data.createdTime.value : this.createdTime,
      updatedTime:
          data.updatedTime.present ? data.updatedTime.value : this.updatedTime,
      size: data.size.present ? data.size.value : this.size,
      description:
          data.description.present ? data.description.value : this.description,
    );
  }

  @override
  String toString() {
    return (StringBuffer('PublicSessionData(')
          ..write('sessionId: $sessionId, ')
          ..write('name: $name, ')
          ..write('avatarKey: $avatarKey, ')
          ..write('createdTime: $createdTime, ')
          ..write('updatedTime: $updatedTime, ')
          ..write('size: $size, ')
          ..write('description: $description')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
      sessionId, name, avatarKey, createdTime, updatedTime, size, description);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is PublicSessionData &&
          other.sessionId == this.sessionId &&
          other.name == this.name &&
          other.avatarKey == this.avatarKey &&
          other.createdTime == this.createdTime &&
          other.updatedTime == this.updatedTime &&
          other.size == this.size &&
          other.description == this.description);
}

class PublicSessionCompanion extends UpdateCompanion<PublicSessionData> {
  final Value<BigInt> sessionId;
  final Value<String> name;
  final Value<String?> avatarKey;
  final Value<int> createdTime;
  final Value<int> updatedTime;
  final Value<int> size;
  final Value<String?> description;
  const PublicSessionCompanion({
    this.sessionId = const Value.absent(),
    this.name = const Value.absent(),
    this.avatarKey = const Value.absent(),
    this.createdTime = const Value.absent(),
    this.updatedTime = const Value.absent(),
    this.size = const Value.absent(),
    this.description = const Value.absent(),
  });
  PublicSessionCompanion.insert({
    this.sessionId = const Value.absent(),
    required String name,
    this.avatarKey = const Value.absent(),
    required int createdTime,
    required int updatedTime,
    required int size,
    this.description = const Value.absent(),
  })  : name = Value(name),
        createdTime = Value(createdTime),
        updatedTime = Value(updatedTime),
        size = Value(size);
  static Insertable<PublicSessionData> custom({
    Expression<BigInt>? sessionId,
    Expression<String>? name,
    Expression<String>? avatarKey,
    Expression<int>? createdTime,
    Expression<int>? updatedTime,
    Expression<int>? size,
    Expression<String>? description,
  }) {
    return RawValuesInsertable({
      if (sessionId != null) 'session_id': sessionId,
      if (name != null) 'name': name,
      if (avatarKey != null) 'avatar_key': avatarKey,
      if (createdTime != null) 'created_time': createdTime,
      if (updatedTime != null) 'updated_time': updatedTime,
      if (size != null) 'size': size,
      if (description != null) 'description': description,
    });
  }

  PublicSessionCompanion copyWith(
      {Value<BigInt>? sessionId,
      Value<String>? name,
      Value<String?>? avatarKey,
      Value<int>? createdTime,
      Value<int>? updatedTime,
      Value<int>? size,
      Value<String?>? description}) {
    return PublicSessionCompanion(
      sessionId: sessionId ?? this.sessionId,
      name: name ?? this.name,
      avatarKey: avatarKey ?? this.avatarKey,
      createdTime: createdTime ?? this.createdTime,
      updatedTime: updatedTime ?? this.updatedTime,
      size: size ?? this.size,
      description: description ?? this.description,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (sessionId.present) {
      map['session_id'] = Variable<BigInt>(sessionId.value);
    }
    if (name.present) {
      map['name'] = Variable<String>(name.value);
    }
    if (avatarKey.present) {
      map['avatar_key'] = Variable<String>(avatarKey.value);
    }
    if (createdTime.present) {
      map['created_time'] = Variable<int>(createdTime.value);
    }
    if (updatedTime.present) {
      map['updated_time'] = Variable<int>(updatedTime.value);
    }
    if (size.present) {
      map['size'] = Variable<int>(size.value);
    }
    if (description.present) {
      map['description'] = Variable<String>(description.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('PublicSessionCompanion(')
          ..write('sessionId: $sessionId, ')
          ..write('name: $name, ')
          ..write('avatarKey: $avatarKey, ')
          ..write('createdTime: $createdTime, ')
          ..write('updatedTime: $updatedTime, ')
          ..write('size: $size, ')
          ..write('description: $description')
          ..write(')'))
        .toString();
  }
}

class $PublicAccountTable extends PublicAccount
    with TableInfo<$PublicAccountTable, PublicAccountData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $PublicAccountTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<BigInt> id = GeneratedColumn<BigInt>(
      'id', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: false);
  static const VerificationMeta _usernameMeta =
      const VerificationMeta('username');
  @override
  late final GeneratedColumn<String> username = GeneratedColumn<String>(
      'username', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _statusMeta = const VerificationMeta('status');
  @override
  late final GeneratedColumn<String> status = GeneratedColumn<String>(
      'status', aliasedName, true,
      type: DriftSqlType.string, requiredDuringInsert: false);
  static const VerificationMeta _avatarKeyMeta =
      const VerificationMeta('avatarKey');
  @override
  late final GeneratedColumn<String> avatarKey = GeneratedColumn<String>(
      'avatar_key', aliasedName, true,
      type: DriftSqlType.string, requiredDuringInsert: false);
  static const VerificationMeta _ocidMeta = const VerificationMeta('ocid');
  @override
  late final GeneratedColumn<String> ocid = GeneratedColumn<String>(
      'ocid', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _publicUpdateTimeMeta =
      const VerificationMeta('publicUpdateTime');
  @override
  late final GeneratedColumn<DateTime> publicUpdateTime =
      GeneratedColumn<DateTime>('public_update_time', aliasedName, false,
          type: DriftSqlType.dateTime, requiredDuringInsert: true);
  @override
  List<GeneratedColumn> get $columns =>
      [id, username, status, avatarKey, ocid, publicUpdateTime];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'public_account';
  @override
  VerificationContext validateIntegrity(Insertable<PublicAccountData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    }
    if (data.containsKey('username')) {
      context.handle(_usernameMeta,
          username.isAcceptableOrUnknown(data['username']!, _usernameMeta));
    } else if (isInserting) {
      context.missing(_usernameMeta);
    }
    if (data.containsKey('status')) {
      context.handle(_statusMeta,
          status.isAcceptableOrUnknown(data['status']!, _statusMeta));
    }
    if (data.containsKey('avatar_key')) {
      context.handle(_avatarKeyMeta,
          avatarKey.isAcceptableOrUnknown(data['avatar_key']!, _avatarKeyMeta));
    }
    if (data.containsKey('ocid')) {
      context.handle(
          _ocidMeta, ocid.isAcceptableOrUnknown(data['ocid']!, _ocidMeta));
    } else if (isInserting) {
      context.missing(_ocidMeta);
    }
    if (data.containsKey('public_update_time')) {
      context.handle(
          _publicUpdateTimeMeta,
          publicUpdateTime.isAcceptableOrUnknown(
              data['public_update_time']!, _publicUpdateTimeMeta));
    } else if (isInserting) {
      context.missing(_publicUpdateTimeMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  PublicAccountData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return PublicAccountData(
      id: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}id'])!,
      username: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}username'])!,
      status: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}status']),
      avatarKey: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}avatar_key']),
      ocid: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}ocid'])!,
      publicUpdateTime: attachedDatabase.typeMapping.read(
          DriftSqlType.dateTime, data['${effectivePrefix}public_update_time'])!,
    );
  }

  @override
  $PublicAccountTable createAlias(String alias) {
    return $PublicAccountTable(attachedDatabase, alias);
  }
}

class PublicAccountData extends DataClass
    implements Insertable<PublicAccountData> {
  final BigInt id;
  final String username;
  final String? status;
  final String? avatarKey;
  final String ocid;
  final DateTime publicUpdateTime;
  const PublicAccountData(
      {required this.id,
      required this.username,
      this.status,
      this.avatarKey,
      required this.ocid,
      required this.publicUpdateTime});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<BigInt>(id);
    map['username'] = Variable<String>(username);
    if (!nullToAbsent || status != null) {
      map['status'] = Variable<String>(status);
    }
    if (!nullToAbsent || avatarKey != null) {
      map['avatar_key'] = Variable<String>(avatarKey);
    }
    map['ocid'] = Variable<String>(ocid);
    map['public_update_time'] = Variable<DateTime>(publicUpdateTime);
    return map;
  }

  PublicAccountCompanion toCompanion(bool nullToAbsent) {
    return PublicAccountCompanion(
      id: Value(id),
      username: Value(username),
      status:
          status == null && nullToAbsent ? const Value.absent() : Value(status),
      avatarKey: avatarKey == null && nullToAbsent
          ? const Value.absent()
          : Value(avatarKey),
      ocid: Value(ocid),
      publicUpdateTime: Value(publicUpdateTime),
    );
  }

  factory PublicAccountData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return PublicAccountData(
      id: serializer.fromJson<BigInt>(json['id']),
      username: serializer.fromJson<String>(json['username']),
      status: serializer.fromJson<String?>(json['status']),
      avatarKey: serializer.fromJson<String?>(json['avatarKey']),
      ocid: serializer.fromJson<String>(json['ocid']),
      publicUpdateTime: serializer.fromJson<DateTime>(json['publicUpdateTime']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<BigInt>(id),
      'username': serializer.toJson<String>(username),
      'status': serializer.toJson<String?>(status),
      'avatarKey': serializer.toJson<String?>(avatarKey),
      'ocid': serializer.toJson<String>(ocid),
      'publicUpdateTime': serializer.toJson<DateTime>(publicUpdateTime),
    };
  }

  PublicAccountData copyWith(
          {BigInt? id,
          String? username,
          Value<String?> status = const Value.absent(),
          Value<String?> avatarKey = const Value.absent(),
          String? ocid,
          DateTime? publicUpdateTime}) =>
      PublicAccountData(
        id: id ?? this.id,
        username: username ?? this.username,
        status: status.present ? status.value : this.status,
        avatarKey: avatarKey.present ? avatarKey.value : this.avatarKey,
        ocid: ocid ?? this.ocid,
        publicUpdateTime: publicUpdateTime ?? this.publicUpdateTime,
      );
  PublicAccountData copyWithCompanion(PublicAccountCompanion data) {
    return PublicAccountData(
      id: data.id.present ? data.id.value : this.id,
      username: data.username.present ? data.username.value : this.username,
      status: data.status.present ? data.status.value : this.status,
      avatarKey: data.avatarKey.present ? data.avatarKey.value : this.avatarKey,
      ocid: data.ocid.present ? data.ocid.value : this.ocid,
      publicUpdateTime: data.publicUpdateTime.present
          ? data.publicUpdateTime.value
          : this.publicUpdateTime,
    );
  }

  @override
  String toString() {
    return (StringBuffer('PublicAccountData(')
          ..write('id: $id, ')
          ..write('username: $username, ')
          ..write('status: $status, ')
          ..write('avatarKey: $avatarKey, ')
          ..write('ocid: $ocid, ')
          ..write('publicUpdateTime: $publicUpdateTime')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode =>
      Object.hash(id, username, status, avatarKey, ocid, publicUpdateTime);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is PublicAccountData &&
          other.id == this.id &&
          other.username == this.username &&
          other.status == this.status &&
          other.avatarKey == this.avatarKey &&
          other.ocid == this.ocid &&
          other.publicUpdateTime == this.publicUpdateTime);
}

class PublicAccountCompanion extends UpdateCompanion<PublicAccountData> {
  final Value<BigInt> id;
  final Value<String> username;
  final Value<String?> status;
  final Value<String?> avatarKey;
  final Value<String> ocid;
  final Value<DateTime> publicUpdateTime;
  const PublicAccountCompanion({
    this.id = const Value.absent(),
    this.username = const Value.absent(),
    this.status = const Value.absent(),
    this.avatarKey = const Value.absent(),
    this.ocid = const Value.absent(),
    this.publicUpdateTime = const Value.absent(),
  });
  PublicAccountCompanion.insert({
    this.id = const Value.absent(),
    required String username,
    this.status = const Value.absent(),
    this.avatarKey = const Value.absent(),
    required String ocid,
    required DateTime publicUpdateTime,
  })  : username = Value(username),
        ocid = Value(ocid),
        publicUpdateTime = Value(publicUpdateTime);
  static Insertable<PublicAccountData> custom({
    Expression<BigInt>? id,
    Expression<String>? username,
    Expression<String>? status,
    Expression<String>? avatarKey,
    Expression<String>? ocid,
    Expression<DateTime>? publicUpdateTime,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (username != null) 'username': username,
      if (status != null) 'status': status,
      if (avatarKey != null) 'avatar_key': avatarKey,
      if (ocid != null) 'ocid': ocid,
      if (publicUpdateTime != null) 'public_update_time': publicUpdateTime,
    });
  }

  PublicAccountCompanion copyWith(
      {Value<BigInt>? id,
      Value<String>? username,
      Value<String?>? status,
      Value<String?>? avatarKey,
      Value<String>? ocid,
      Value<DateTime>? publicUpdateTime}) {
    return PublicAccountCompanion(
      id: id ?? this.id,
      username: username ?? this.username,
      status: status ?? this.status,
      avatarKey: avatarKey ?? this.avatarKey,
      ocid: ocid ?? this.ocid,
      publicUpdateTime: publicUpdateTime ?? this.publicUpdateTime,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<BigInt>(id.value);
    }
    if (username.present) {
      map['username'] = Variable<String>(username.value);
    }
    if (status.present) {
      map['status'] = Variable<String>(status.value);
    }
    if (avatarKey.present) {
      map['avatar_key'] = Variable<String>(avatarKey.value);
    }
    if (ocid.present) {
      map['ocid'] = Variable<String>(ocid.value);
    }
    if (publicUpdateTime.present) {
      map['public_update_time'] = Variable<DateTime>(publicUpdateTime.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('PublicAccountCompanion(')
          ..write('id: $id, ')
          ..write('username: $username, ')
          ..write('status: $status, ')
          ..write('avatarKey: $avatarKey, ')
          ..write('ocid: $ocid, ')
          ..write('publicUpdateTime: $publicUpdateTime')
          ..write(')'))
        .toString();
  }
}

abstract class _$PublicOurchatDatabase extends GeneratedDatabase {
  _$PublicOurchatDatabase(QueryExecutor e) : super(e);
  $PublicOurchatDatabaseManager get managers =>
      $PublicOurchatDatabaseManager(this);
  late final $PublicSessionTable publicSession = $PublicSessionTable(this);
  late final $PublicAccountTable publicAccount = $PublicAccountTable(this);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities =>
      [publicSession, publicAccount];
}

typedef $$PublicSessionTableCreateCompanionBuilder = PublicSessionCompanion
    Function({
  Value<BigInt> sessionId,
  required String name,
  Value<String?> avatarKey,
  required int createdTime,
  required int updatedTime,
  required int size,
  Value<String?> description,
});
typedef $$PublicSessionTableUpdateCompanionBuilder = PublicSessionCompanion
    Function({
  Value<BigInt> sessionId,
  Value<String> name,
  Value<String?> avatarKey,
  Value<int> createdTime,
  Value<int> updatedTime,
  Value<int> size,
  Value<String?> description,
});

class $$PublicSessionTableFilterComposer
    extends Composer<_$PublicOurchatDatabase, $PublicSessionTable> {
  $$PublicSessionTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<BigInt> get sessionId => $composableBuilder(
      column: $table.sessionId, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get name => $composableBuilder(
      column: $table.name, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get avatarKey => $composableBuilder(
      column: $table.avatarKey, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get createdTime => $composableBuilder(
      column: $table.createdTime, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get updatedTime => $composableBuilder(
      column: $table.updatedTime, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get size => $composableBuilder(
      column: $table.size, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get description => $composableBuilder(
      column: $table.description, builder: (column) => ColumnFilters(column));
}

class $$PublicSessionTableOrderingComposer
    extends Composer<_$PublicOurchatDatabase, $PublicSessionTable> {
  $$PublicSessionTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<BigInt> get sessionId => $composableBuilder(
      column: $table.sessionId, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get name => $composableBuilder(
      column: $table.name, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get avatarKey => $composableBuilder(
      column: $table.avatarKey, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get createdTime => $composableBuilder(
      column: $table.createdTime, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get updatedTime => $composableBuilder(
      column: $table.updatedTime, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get size => $composableBuilder(
      column: $table.size, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get description => $composableBuilder(
      column: $table.description, builder: (column) => ColumnOrderings(column));
}

class $$PublicSessionTableAnnotationComposer
    extends Composer<_$PublicOurchatDatabase, $PublicSessionTable> {
  $$PublicSessionTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<BigInt> get sessionId =>
      $composableBuilder(column: $table.sessionId, builder: (column) => column);

  GeneratedColumn<String> get name =>
      $composableBuilder(column: $table.name, builder: (column) => column);

  GeneratedColumn<String> get avatarKey =>
      $composableBuilder(column: $table.avatarKey, builder: (column) => column);

  GeneratedColumn<int> get createdTime => $composableBuilder(
      column: $table.createdTime, builder: (column) => column);

  GeneratedColumn<int> get updatedTime => $composableBuilder(
      column: $table.updatedTime, builder: (column) => column);

  GeneratedColumn<int> get size =>
      $composableBuilder(column: $table.size, builder: (column) => column);

  GeneratedColumn<String> get description => $composableBuilder(
      column: $table.description, builder: (column) => column);
}

class $$PublicSessionTableTableManager extends RootTableManager<
    _$PublicOurchatDatabase,
    $PublicSessionTable,
    PublicSessionData,
    $$PublicSessionTableFilterComposer,
    $$PublicSessionTableOrderingComposer,
    $$PublicSessionTableAnnotationComposer,
    $$PublicSessionTableCreateCompanionBuilder,
    $$PublicSessionTableUpdateCompanionBuilder,
    (
      PublicSessionData,
      BaseReferences<_$PublicOurchatDatabase, $PublicSessionTable,
          PublicSessionData>
    ),
    PublicSessionData,
    PrefetchHooks Function()> {
  $$PublicSessionTableTableManager(
      _$PublicOurchatDatabase db, $PublicSessionTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$PublicSessionTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$PublicSessionTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$PublicSessionTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<BigInt> sessionId = const Value.absent(),
            Value<String> name = const Value.absent(),
            Value<String?> avatarKey = const Value.absent(),
            Value<int> createdTime = const Value.absent(),
            Value<int> updatedTime = const Value.absent(),
            Value<int> size = const Value.absent(),
            Value<String?> description = const Value.absent(),
          }) =>
              PublicSessionCompanion(
            sessionId: sessionId,
            name: name,
            avatarKey: avatarKey,
            createdTime: createdTime,
            updatedTime: updatedTime,
            size: size,
            description: description,
          ),
          createCompanionCallback: ({
            Value<BigInt> sessionId = const Value.absent(),
            required String name,
            Value<String?> avatarKey = const Value.absent(),
            required int createdTime,
            required int updatedTime,
            required int size,
            Value<String?> description = const Value.absent(),
          }) =>
              PublicSessionCompanion.insert(
            sessionId: sessionId,
            name: name,
            avatarKey: avatarKey,
            createdTime: createdTime,
            updatedTime: updatedTime,
            size: size,
            description: description,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$PublicSessionTableProcessedTableManager = ProcessedTableManager<
    _$PublicOurchatDatabase,
    $PublicSessionTable,
    PublicSessionData,
    $$PublicSessionTableFilterComposer,
    $$PublicSessionTableOrderingComposer,
    $$PublicSessionTableAnnotationComposer,
    $$PublicSessionTableCreateCompanionBuilder,
    $$PublicSessionTableUpdateCompanionBuilder,
    (
      PublicSessionData,
      BaseReferences<_$PublicOurchatDatabase, $PublicSessionTable,
          PublicSessionData>
    ),
    PublicSessionData,
    PrefetchHooks Function()>;
typedef $$PublicAccountTableCreateCompanionBuilder = PublicAccountCompanion
    Function({
  Value<BigInt> id,
  required String username,
  Value<String?> status,
  Value<String?> avatarKey,
  required String ocid,
  required DateTime publicUpdateTime,
});
typedef $$PublicAccountTableUpdateCompanionBuilder = PublicAccountCompanion
    Function({
  Value<BigInt> id,
  Value<String> username,
  Value<String?> status,
  Value<String?> avatarKey,
  Value<String> ocid,
  Value<DateTime> publicUpdateTime,
});

class $$PublicAccountTableFilterComposer
    extends Composer<_$PublicOurchatDatabase, $PublicAccountTable> {
  $$PublicAccountTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<BigInt> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get username => $composableBuilder(
      column: $table.username, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get status => $composableBuilder(
      column: $table.status, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get avatarKey => $composableBuilder(
      column: $table.avatarKey, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get ocid => $composableBuilder(
      column: $table.ocid, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get publicUpdateTime => $composableBuilder(
      column: $table.publicUpdateTime,
      builder: (column) => ColumnFilters(column));
}

class $$PublicAccountTableOrderingComposer
    extends Composer<_$PublicOurchatDatabase, $PublicAccountTable> {
  $$PublicAccountTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<BigInt> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get username => $composableBuilder(
      column: $table.username, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get status => $composableBuilder(
      column: $table.status, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get avatarKey => $composableBuilder(
      column: $table.avatarKey, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get ocid => $composableBuilder(
      column: $table.ocid, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get publicUpdateTime => $composableBuilder(
      column: $table.publicUpdateTime,
      builder: (column) => ColumnOrderings(column));
}

class $$PublicAccountTableAnnotationComposer
    extends Composer<_$PublicOurchatDatabase, $PublicAccountTable> {
  $$PublicAccountTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<BigInt> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get username =>
      $composableBuilder(column: $table.username, builder: (column) => column);

  GeneratedColumn<String> get status =>
      $composableBuilder(column: $table.status, builder: (column) => column);

  GeneratedColumn<String> get avatarKey =>
      $composableBuilder(column: $table.avatarKey, builder: (column) => column);

  GeneratedColumn<String> get ocid =>
      $composableBuilder(column: $table.ocid, builder: (column) => column);

  GeneratedColumn<DateTime> get publicUpdateTime => $composableBuilder(
      column: $table.publicUpdateTime, builder: (column) => column);
}

class $$PublicAccountTableTableManager extends RootTableManager<
    _$PublicOurchatDatabase,
    $PublicAccountTable,
    PublicAccountData,
    $$PublicAccountTableFilterComposer,
    $$PublicAccountTableOrderingComposer,
    $$PublicAccountTableAnnotationComposer,
    $$PublicAccountTableCreateCompanionBuilder,
    $$PublicAccountTableUpdateCompanionBuilder,
    (
      PublicAccountData,
      BaseReferences<_$PublicOurchatDatabase, $PublicAccountTable,
          PublicAccountData>
    ),
    PublicAccountData,
    PrefetchHooks Function()> {
  $$PublicAccountTableTableManager(
      _$PublicOurchatDatabase db, $PublicAccountTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$PublicAccountTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$PublicAccountTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$PublicAccountTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<BigInt> id = const Value.absent(),
            Value<String> username = const Value.absent(),
            Value<String?> status = const Value.absent(),
            Value<String?> avatarKey = const Value.absent(),
            Value<String> ocid = const Value.absent(),
            Value<DateTime> publicUpdateTime = const Value.absent(),
          }) =>
              PublicAccountCompanion(
            id: id,
            username: username,
            status: status,
            avatarKey: avatarKey,
            ocid: ocid,
            publicUpdateTime: publicUpdateTime,
          ),
          createCompanionCallback: ({
            Value<BigInt> id = const Value.absent(),
            required String username,
            Value<String?> status = const Value.absent(),
            Value<String?> avatarKey = const Value.absent(),
            required String ocid,
            required DateTime publicUpdateTime,
          }) =>
              PublicAccountCompanion.insert(
            id: id,
            username: username,
            status: status,
            avatarKey: avatarKey,
            ocid: ocid,
            publicUpdateTime: publicUpdateTime,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$PublicAccountTableProcessedTableManager = ProcessedTableManager<
    _$PublicOurchatDatabase,
    $PublicAccountTable,
    PublicAccountData,
    $$PublicAccountTableFilterComposer,
    $$PublicAccountTableOrderingComposer,
    $$PublicAccountTableAnnotationComposer,
    $$PublicAccountTableCreateCompanionBuilder,
    $$PublicAccountTableUpdateCompanionBuilder,
    (
      PublicAccountData,
      BaseReferences<_$PublicOurchatDatabase, $PublicAccountTable,
          PublicAccountData>
    ),
    PublicAccountData,
    PrefetchHooks Function()>;

class $PublicOurchatDatabaseManager {
  final _$PublicOurchatDatabase _db;
  $PublicOurchatDatabaseManager(this._db);
  $$PublicSessionTableTableManager get publicSession =>
      $$PublicSessionTableTableManager(_db, _db.publicSession);
  $$PublicAccountTableTableManager get publicAccount =>
      $$PublicAccountTableTableManager(_db, _db.publicAccount);
}

class $AccountTable extends Account with TableInfo<$AccountTable, AccountData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $AccountTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<BigInt> id = GeneratedColumn<BigInt>(
      'id', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: true);
  static const VerificationMeta _emailMeta = const VerificationMeta('email');
  @override
  late final GeneratedColumn<String> email = GeneratedColumn<String>(
      'email', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _registerTimeMeta =
      const VerificationMeta('registerTime');
  @override
  late final GeneratedColumn<DateTime> registerTime = GeneratedColumn<DateTime>(
      'register_time', aliasedName, false,
      type: DriftSqlType.dateTime, requiredDuringInsert: true);
  static const VerificationMeta _updateTimeMeta =
      const VerificationMeta('updateTime');
  @override
  late final GeneratedColumn<DateTime> updateTime = GeneratedColumn<DateTime>(
      'update_time', aliasedName, false,
      type: DriftSqlType.dateTime, requiredDuringInsert: true);
  static const VerificationMeta _friendsJsonMeta =
      const VerificationMeta('friendsJson');
  @override
  late final GeneratedColumn<String> friendsJson = GeneratedColumn<String>(
      'friends_json', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _sessionsJsonMeta =
      const VerificationMeta('sessionsJson');
  @override
  late final GeneratedColumn<String> sessionsJson = GeneratedColumn<String>(
      'sessions_json', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _latestMsgTimeMeta =
      const VerificationMeta('latestMsgTime');
  @override
  late final GeneratedColumn<DateTime> latestMsgTime =
      GeneratedColumn<DateTime>('latest_msg_time', aliasedName, false,
          type: DriftSqlType.dateTime, requiredDuringInsert: true);
  @override
  List<GeneratedColumn> get $columns => [
        id,
        email,
        registerTime,
        updateTime,
        friendsJson,
        sessionsJson,
        latestMsgTime
      ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'account';
  @override
  VerificationContext validateIntegrity(Insertable<AccountData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('email')) {
      context.handle(
          _emailMeta, email.isAcceptableOrUnknown(data['email']!, _emailMeta));
    } else if (isInserting) {
      context.missing(_emailMeta);
    }
    if (data.containsKey('register_time')) {
      context.handle(
          _registerTimeMeta,
          registerTime.isAcceptableOrUnknown(
              data['register_time']!, _registerTimeMeta));
    } else if (isInserting) {
      context.missing(_registerTimeMeta);
    }
    if (data.containsKey('update_time')) {
      context.handle(
          _updateTimeMeta,
          updateTime.isAcceptableOrUnknown(
              data['update_time']!, _updateTimeMeta));
    } else if (isInserting) {
      context.missing(_updateTimeMeta);
    }
    if (data.containsKey('friends_json')) {
      context.handle(
          _friendsJsonMeta,
          friendsJson.isAcceptableOrUnknown(
              data['friends_json']!, _friendsJsonMeta));
    } else if (isInserting) {
      context.missing(_friendsJsonMeta);
    }
    if (data.containsKey('sessions_json')) {
      context.handle(
          _sessionsJsonMeta,
          sessionsJson.isAcceptableOrUnknown(
              data['sessions_json']!, _sessionsJsonMeta));
    } else if (isInserting) {
      context.missing(_sessionsJsonMeta);
    }
    if (data.containsKey('latest_msg_time')) {
      context.handle(
          _latestMsgTimeMeta,
          latestMsgTime.isAcceptableOrUnknown(
              data['latest_msg_time']!, _latestMsgTimeMeta));
    } else if (isInserting) {
      context.missing(_latestMsgTimeMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => const {};
  @override
  AccountData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return AccountData(
      id: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}id'])!,
      email: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}email'])!,
      registerTime: attachedDatabase.typeMapping.read(
          DriftSqlType.dateTime, data['${effectivePrefix}register_time'])!,
      updateTime: attachedDatabase.typeMapping
          .read(DriftSqlType.dateTime, data['${effectivePrefix}update_time'])!,
      friendsJson: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}friends_json'])!,
      sessionsJson: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}sessions_json'])!,
      latestMsgTime: attachedDatabase.typeMapping.read(
          DriftSqlType.dateTime, data['${effectivePrefix}latest_msg_time'])!,
    );
  }

  @override
  $AccountTable createAlias(String alias) {
    return $AccountTable(attachedDatabase, alias);
  }
}

class AccountData extends DataClass implements Insertable<AccountData> {
  final BigInt id;
  final String email;
  final DateTime registerTime;
  final DateTime updateTime;
  final String friendsJson;
  final String sessionsJson;
  final DateTime latestMsgTime;
  const AccountData(
      {required this.id,
      required this.email,
      required this.registerTime,
      required this.updateTime,
      required this.friendsJson,
      required this.sessionsJson,
      required this.latestMsgTime});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<BigInt>(id);
    map['email'] = Variable<String>(email);
    map['register_time'] = Variable<DateTime>(registerTime);
    map['update_time'] = Variable<DateTime>(updateTime);
    map['friends_json'] = Variable<String>(friendsJson);
    map['sessions_json'] = Variable<String>(sessionsJson);
    map['latest_msg_time'] = Variable<DateTime>(latestMsgTime);
    return map;
  }

  AccountCompanion toCompanion(bool nullToAbsent) {
    return AccountCompanion(
      id: Value(id),
      email: Value(email),
      registerTime: Value(registerTime),
      updateTime: Value(updateTime),
      friendsJson: Value(friendsJson),
      sessionsJson: Value(sessionsJson),
      latestMsgTime: Value(latestMsgTime),
    );
  }

  factory AccountData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return AccountData(
      id: serializer.fromJson<BigInt>(json['id']),
      email: serializer.fromJson<String>(json['email']),
      registerTime: serializer.fromJson<DateTime>(json['registerTime']),
      updateTime: serializer.fromJson<DateTime>(json['updateTime']),
      friendsJson: serializer.fromJson<String>(json['friendsJson']),
      sessionsJson: serializer.fromJson<String>(json['sessionsJson']),
      latestMsgTime: serializer.fromJson<DateTime>(json['latestMsgTime']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<BigInt>(id),
      'email': serializer.toJson<String>(email),
      'registerTime': serializer.toJson<DateTime>(registerTime),
      'updateTime': serializer.toJson<DateTime>(updateTime),
      'friendsJson': serializer.toJson<String>(friendsJson),
      'sessionsJson': serializer.toJson<String>(sessionsJson),
      'latestMsgTime': serializer.toJson<DateTime>(latestMsgTime),
    };
  }

  AccountData copyWith(
          {BigInt? id,
          String? email,
          DateTime? registerTime,
          DateTime? updateTime,
          String? friendsJson,
          String? sessionsJson,
          DateTime? latestMsgTime}) =>
      AccountData(
        id: id ?? this.id,
        email: email ?? this.email,
        registerTime: registerTime ?? this.registerTime,
        updateTime: updateTime ?? this.updateTime,
        friendsJson: friendsJson ?? this.friendsJson,
        sessionsJson: sessionsJson ?? this.sessionsJson,
        latestMsgTime: latestMsgTime ?? this.latestMsgTime,
      );
  AccountData copyWithCompanion(AccountCompanion data) {
    return AccountData(
      id: data.id.present ? data.id.value : this.id,
      email: data.email.present ? data.email.value : this.email,
      registerTime: data.registerTime.present
          ? data.registerTime.value
          : this.registerTime,
      updateTime:
          data.updateTime.present ? data.updateTime.value : this.updateTime,
      friendsJson:
          data.friendsJson.present ? data.friendsJson.value : this.friendsJson,
      sessionsJson: data.sessionsJson.present
          ? data.sessionsJson.value
          : this.sessionsJson,
      latestMsgTime: data.latestMsgTime.present
          ? data.latestMsgTime.value
          : this.latestMsgTime,
    );
  }

  @override
  String toString() {
    return (StringBuffer('AccountData(')
          ..write('id: $id, ')
          ..write('email: $email, ')
          ..write('registerTime: $registerTime, ')
          ..write('updateTime: $updateTime, ')
          ..write('friendsJson: $friendsJson, ')
          ..write('sessionsJson: $sessionsJson, ')
          ..write('latestMsgTime: $latestMsgTime')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(id, email, registerTime, updateTime,
      friendsJson, sessionsJson, latestMsgTime);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is AccountData &&
          other.id == this.id &&
          other.email == this.email &&
          other.registerTime == this.registerTime &&
          other.updateTime == this.updateTime &&
          other.friendsJson == this.friendsJson &&
          other.sessionsJson == this.sessionsJson &&
          other.latestMsgTime == this.latestMsgTime);
}

class AccountCompanion extends UpdateCompanion<AccountData> {
  final Value<BigInt> id;
  final Value<String> email;
  final Value<DateTime> registerTime;
  final Value<DateTime> updateTime;
  final Value<String> friendsJson;
  final Value<String> sessionsJson;
  final Value<DateTime> latestMsgTime;
  final Value<int> rowid;
  const AccountCompanion({
    this.id = const Value.absent(),
    this.email = const Value.absent(),
    this.registerTime = const Value.absent(),
    this.updateTime = const Value.absent(),
    this.friendsJson = const Value.absent(),
    this.sessionsJson = const Value.absent(),
    this.latestMsgTime = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  AccountCompanion.insert({
    required BigInt id,
    required String email,
    required DateTime registerTime,
    required DateTime updateTime,
    required String friendsJson,
    required String sessionsJson,
    required DateTime latestMsgTime,
    this.rowid = const Value.absent(),
  })  : id = Value(id),
        email = Value(email),
        registerTime = Value(registerTime),
        updateTime = Value(updateTime),
        friendsJson = Value(friendsJson),
        sessionsJson = Value(sessionsJson),
        latestMsgTime = Value(latestMsgTime);
  static Insertable<AccountData> custom({
    Expression<BigInt>? id,
    Expression<String>? email,
    Expression<DateTime>? registerTime,
    Expression<DateTime>? updateTime,
    Expression<String>? friendsJson,
    Expression<String>? sessionsJson,
    Expression<DateTime>? latestMsgTime,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (email != null) 'email': email,
      if (registerTime != null) 'register_time': registerTime,
      if (updateTime != null) 'update_time': updateTime,
      if (friendsJson != null) 'friends_json': friendsJson,
      if (sessionsJson != null) 'sessions_json': sessionsJson,
      if (latestMsgTime != null) 'latest_msg_time': latestMsgTime,
      if (rowid != null) 'rowid': rowid,
    });
  }

  AccountCompanion copyWith(
      {Value<BigInt>? id,
      Value<String>? email,
      Value<DateTime>? registerTime,
      Value<DateTime>? updateTime,
      Value<String>? friendsJson,
      Value<String>? sessionsJson,
      Value<DateTime>? latestMsgTime,
      Value<int>? rowid}) {
    return AccountCompanion(
      id: id ?? this.id,
      email: email ?? this.email,
      registerTime: registerTime ?? this.registerTime,
      updateTime: updateTime ?? this.updateTime,
      friendsJson: friendsJson ?? this.friendsJson,
      sessionsJson: sessionsJson ?? this.sessionsJson,
      latestMsgTime: latestMsgTime ?? this.latestMsgTime,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<BigInt>(id.value);
    }
    if (email.present) {
      map['email'] = Variable<String>(email.value);
    }
    if (registerTime.present) {
      map['register_time'] = Variable<DateTime>(registerTime.value);
    }
    if (updateTime.present) {
      map['update_time'] = Variable<DateTime>(updateTime.value);
    }
    if (friendsJson.present) {
      map['friends_json'] = Variable<String>(friendsJson.value);
    }
    if (sessionsJson.present) {
      map['sessions_json'] = Variable<String>(sessionsJson.value);
    }
    if (latestMsgTime.present) {
      map['latest_msg_time'] = Variable<DateTime>(latestMsgTime.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('AccountCompanion(')
          ..write('id: $id, ')
          ..write('email: $email, ')
          ..write('registerTime: $registerTime, ')
          ..write('updateTime: $updateTime, ')
          ..write('friendsJson: $friendsJson, ')
          ..write('sessionsJson: $sessionsJson, ')
          ..write('latestMsgTime: $latestMsgTime, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $SessionTable extends Session with TableInfo<$SessionTable, SessionData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $SessionTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _sessionIdMeta =
      const VerificationMeta('sessionId');
  @override
  late final GeneratedColumn<BigInt> sessionId = GeneratedColumn<BigInt>(
      'session_id', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: false);
  static const VerificationMeta _membersMeta =
      const VerificationMeta('members');
  @override
  late final GeneratedColumn<String> members = GeneratedColumn<String>(
      'members', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _rolesMeta = const VerificationMeta('roles');
  @override
  late final GeneratedColumn<String> roles = GeneratedColumn<String>(
      'roles', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  @override
  List<GeneratedColumn> get $columns => [sessionId, members, roles];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'session';
  @override
  VerificationContext validateIntegrity(Insertable<SessionData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('session_id')) {
      context.handle(_sessionIdMeta,
          sessionId.isAcceptableOrUnknown(data['session_id']!, _sessionIdMeta));
    }
    if (data.containsKey('members')) {
      context.handle(_membersMeta,
          members.isAcceptableOrUnknown(data['members']!, _membersMeta));
    } else if (isInserting) {
      context.missing(_membersMeta);
    }
    if (data.containsKey('roles')) {
      context.handle(
          _rolesMeta, roles.isAcceptableOrUnknown(data['roles']!, _rolesMeta));
    } else if (isInserting) {
      context.missing(_rolesMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {sessionId};
  @override
  SessionData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return SessionData(
      sessionId: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}session_id'])!,
      members: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}members'])!,
      roles: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}roles'])!,
    );
  }

  @override
  $SessionTable createAlias(String alias) {
    return $SessionTable(attachedDatabase, alias);
  }
}

class SessionData extends DataClass implements Insertable<SessionData> {
  final BigInt sessionId;
  final String members;
  final String roles;
  const SessionData(
      {required this.sessionId, required this.members, required this.roles});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['session_id'] = Variable<BigInt>(sessionId);
    map['members'] = Variable<String>(members);
    map['roles'] = Variable<String>(roles);
    return map;
  }

  SessionCompanion toCompanion(bool nullToAbsent) {
    return SessionCompanion(
      sessionId: Value(sessionId),
      members: Value(members),
      roles: Value(roles),
    );
  }

  factory SessionData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return SessionData(
      sessionId: serializer.fromJson<BigInt>(json['sessionId']),
      members: serializer.fromJson<String>(json['members']),
      roles: serializer.fromJson<String>(json['roles']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'sessionId': serializer.toJson<BigInt>(sessionId),
      'members': serializer.toJson<String>(members),
      'roles': serializer.toJson<String>(roles),
    };
  }

  SessionData copyWith({BigInt? sessionId, String? members, String? roles}) =>
      SessionData(
        sessionId: sessionId ?? this.sessionId,
        members: members ?? this.members,
        roles: roles ?? this.roles,
      );
  SessionData copyWithCompanion(SessionCompanion data) {
    return SessionData(
      sessionId: data.sessionId.present ? data.sessionId.value : this.sessionId,
      members: data.members.present ? data.members.value : this.members,
      roles: data.roles.present ? data.roles.value : this.roles,
    );
  }

  @override
  String toString() {
    return (StringBuffer('SessionData(')
          ..write('sessionId: $sessionId, ')
          ..write('members: $members, ')
          ..write('roles: $roles')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(sessionId, members, roles);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is SessionData &&
          other.sessionId == this.sessionId &&
          other.members == this.members &&
          other.roles == this.roles);
}

class SessionCompanion extends UpdateCompanion<SessionData> {
  final Value<BigInt> sessionId;
  final Value<String> members;
  final Value<String> roles;
  const SessionCompanion({
    this.sessionId = const Value.absent(),
    this.members = const Value.absent(),
    this.roles = const Value.absent(),
  });
  SessionCompanion.insert({
    this.sessionId = const Value.absent(),
    required String members,
    required String roles,
  })  : members = Value(members),
        roles = Value(roles);
  static Insertable<SessionData> custom({
    Expression<BigInt>? sessionId,
    Expression<String>? members,
    Expression<String>? roles,
  }) {
    return RawValuesInsertable({
      if (sessionId != null) 'session_id': sessionId,
      if (members != null) 'members': members,
      if (roles != null) 'roles': roles,
    });
  }

  SessionCompanion copyWith(
      {Value<BigInt>? sessionId,
      Value<String>? members,
      Value<String>? roles}) {
    return SessionCompanion(
      sessionId: sessionId ?? this.sessionId,
      members: members ?? this.members,
      roles: roles ?? this.roles,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (sessionId.present) {
      map['session_id'] = Variable<BigInt>(sessionId.value);
    }
    if (members.present) {
      map['members'] = Variable<String>(members.value);
    }
    if (roles.present) {
      map['roles'] = Variable<String>(roles.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('SessionCompanion(')
          ..write('sessionId: $sessionId, ')
          ..write('members: $members, ')
          ..write('roles: $roles')
          ..write(')'))
        .toString();
  }
}

class $RecordTable extends Record with TableInfo<$RecordTable, RecordData> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $RecordTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _msgIdMeta = const VerificationMeta('msgId');
  @override
  late final GeneratedColumn<BigInt> msgId = GeneratedColumn<BigInt>(
      'msg_id', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: false);
  static const VerificationMeta _fromSessionMeta =
      const VerificationMeta('fromSession');
  @override
  late final GeneratedColumn<int> fromSession = GeneratedColumn<int>(
      'from_session', aliasedName, true,
      type: DriftSqlType.int, requiredDuringInsert: false);
  static const VerificationMeta _senderMeta = const VerificationMeta('sender');
  @override
  late final GeneratedColumn<BigInt> sender = GeneratedColumn<BigInt>(
      'sender', aliasedName, false,
      type: DriftSqlType.bigInt, requiredDuringInsert: true);
  static const VerificationMeta _timeMeta = const VerificationMeta('time');
  @override
  late final GeneratedColumn<DateTime> time = GeneratedColumn<DateTime>(
      'time', aliasedName, false,
      type: DriftSqlType.dateTime, requiredDuringInsert: true);
  static const VerificationMeta _dataMeta = const VerificationMeta('data');
  @override
  late final GeneratedColumn<String> data = GeneratedColumn<String>(
      'data', aliasedName, false,
      type: DriftSqlType.string, requiredDuringInsert: true);
  static const VerificationMeta _readMeta = const VerificationMeta('read');
  @override
  late final GeneratedColumn<int> read = GeneratedColumn<int>(
      'read', aliasedName, false,
      type: DriftSqlType.int,
      requiredDuringInsert: false,
      defaultValue: const Constant(0));
  @override
  List<GeneratedColumn> get $columns =>
      [msgId, fromSession, sender, time, data, read];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'record';
  @override
  VerificationContext validateIntegrity(Insertable<RecordData> instance,
      {bool isInserting = false}) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('msg_id')) {
      context.handle(
          _msgIdMeta, msgId.isAcceptableOrUnknown(data['msg_id']!, _msgIdMeta));
    }
    if (data.containsKey('from_session')) {
      context.handle(
          _fromSessionMeta,
          fromSession.isAcceptableOrUnknown(
              data['from_session']!, _fromSessionMeta));
    }
    if (data.containsKey('sender')) {
      context.handle(_senderMeta,
          sender.isAcceptableOrUnknown(data['sender']!, _senderMeta));
    } else if (isInserting) {
      context.missing(_senderMeta);
    }
    if (data.containsKey('time')) {
      context.handle(
          _timeMeta, time.isAcceptableOrUnknown(data['time']!, _timeMeta));
    } else if (isInserting) {
      context.missing(_timeMeta);
    }
    if (data.containsKey('data')) {
      context.handle(
          _dataMeta, this.data.isAcceptableOrUnknown(data['data']!, _dataMeta));
    } else if (isInserting) {
      context.missing(_dataMeta);
    }
    if (data.containsKey('read')) {
      context.handle(
          _readMeta, read.isAcceptableOrUnknown(data['read']!, _readMeta));
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {msgId};
  @override
  RecordData map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return RecordData(
      msgId: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}msg_id'])!,
      fromSession: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}from_session']),
      sender: attachedDatabase.typeMapping
          .read(DriftSqlType.bigInt, data['${effectivePrefix}sender'])!,
      time: attachedDatabase.typeMapping
          .read(DriftSqlType.dateTime, data['${effectivePrefix}time'])!,
      data: attachedDatabase.typeMapping
          .read(DriftSqlType.string, data['${effectivePrefix}data'])!,
      read: attachedDatabase.typeMapping
          .read(DriftSqlType.int, data['${effectivePrefix}read'])!,
    );
  }

  @override
  $RecordTable createAlias(String alias) {
    return $RecordTable(attachedDatabase, alias);
  }
}

class RecordData extends DataClass implements Insertable<RecordData> {
  final BigInt msgId;
  final int? fromSession;
  final BigInt sender;
  final DateTime time;
  final String data;
  final int read;
  const RecordData(
      {required this.msgId,
      this.fromSession,
      required this.sender,
      required this.time,
      required this.data,
      required this.read});
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['msg_id'] = Variable<BigInt>(msgId);
    if (!nullToAbsent || fromSession != null) {
      map['from_session'] = Variable<int>(fromSession);
    }
    map['sender'] = Variable<BigInt>(sender);
    map['time'] = Variable<DateTime>(time);
    map['data'] = Variable<String>(data);
    map['read'] = Variable<int>(read);
    return map;
  }

  RecordCompanion toCompanion(bool nullToAbsent) {
    return RecordCompanion(
      msgId: Value(msgId),
      fromSession: fromSession == null && nullToAbsent
          ? const Value.absent()
          : Value(fromSession),
      sender: Value(sender),
      time: Value(time),
      data: Value(data),
      read: Value(read),
    );
  }

  factory RecordData.fromJson(Map<String, dynamic> json,
      {ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return RecordData(
      msgId: serializer.fromJson<BigInt>(json['msgId']),
      fromSession: serializer.fromJson<int?>(json['fromSession']),
      sender: serializer.fromJson<BigInt>(json['sender']),
      time: serializer.fromJson<DateTime>(json['time']),
      data: serializer.fromJson<String>(json['data']),
      read: serializer.fromJson<int>(json['read']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'msgId': serializer.toJson<BigInt>(msgId),
      'fromSession': serializer.toJson<int?>(fromSession),
      'sender': serializer.toJson<BigInt>(sender),
      'time': serializer.toJson<DateTime>(time),
      'data': serializer.toJson<String>(data),
      'read': serializer.toJson<int>(read),
    };
  }

  RecordData copyWith(
          {BigInt? msgId,
          Value<int?> fromSession = const Value.absent(),
          BigInt? sender,
          DateTime? time,
          String? data,
          int? read}) =>
      RecordData(
        msgId: msgId ?? this.msgId,
        fromSession: fromSession.present ? fromSession.value : this.fromSession,
        sender: sender ?? this.sender,
        time: time ?? this.time,
        data: data ?? this.data,
        read: read ?? this.read,
      );
  RecordData copyWithCompanion(RecordCompanion data) {
    return RecordData(
      msgId: data.msgId.present ? data.msgId.value : this.msgId,
      fromSession:
          data.fromSession.present ? data.fromSession.value : this.fromSession,
      sender: data.sender.present ? data.sender.value : this.sender,
      time: data.time.present ? data.time.value : this.time,
      data: data.data.present ? data.data.value : this.data,
      read: data.read.present ? data.read.value : this.read,
    );
  }

  @override
  String toString() {
    return (StringBuffer('RecordData(')
          ..write('msgId: $msgId, ')
          ..write('fromSession: $fromSession, ')
          ..write('sender: $sender, ')
          ..write('time: $time, ')
          ..write('data: $data, ')
          ..write('read: $read')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(msgId, fromSession, sender, time, data, read);
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is RecordData &&
          other.msgId == this.msgId &&
          other.fromSession == this.fromSession &&
          other.sender == this.sender &&
          other.time == this.time &&
          other.data == this.data &&
          other.read == this.read);
}

class RecordCompanion extends UpdateCompanion<RecordData> {
  final Value<BigInt> msgId;
  final Value<int?> fromSession;
  final Value<BigInt> sender;
  final Value<DateTime> time;
  final Value<String> data;
  final Value<int> read;
  const RecordCompanion({
    this.msgId = const Value.absent(),
    this.fromSession = const Value.absent(),
    this.sender = const Value.absent(),
    this.time = const Value.absent(),
    this.data = const Value.absent(),
    this.read = const Value.absent(),
  });
  RecordCompanion.insert({
    this.msgId = const Value.absent(),
    this.fromSession = const Value.absent(),
    required BigInt sender,
    required DateTime time,
    required String data,
    this.read = const Value.absent(),
  })  : sender = Value(sender),
        time = Value(time),
        data = Value(data);
  static Insertable<RecordData> custom({
    Expression<BigInt>? msgId,
    Expression<int>? fromSession,
    Expression<BigInt>? sender,
    Expression<DateTime>? time,
    Expression<String>? data,
    Expression<int>? read,
  }) {
    return RawValuesInsertable({
      if (msgId != null) 'msg_id': msgId,
      if (fromSession != null) 'from_session': fromSession,
      if (sender != null) 'sender': sender,
      if (time != null) 'time': time,
      if (data != null) 'data': data,
      if (read != null) 'read': read,
    });
  }

  RecordCompanion copyWith(
      {Value<BigInt>? msgId,
      Value<int?>? fromSession,
      Value<BigInt>? sender,
      Value<DateTime>? time,
      Value<String>? data,
      Value<int>? read}) {
    return RecordCompanion(
      msgId: msgId ?? this.msgId,
      fromSession: fromSession ?? this.fromSession,
      sender: sender ?? this.sender,
      time: time ?? this.time,
      data: data ?? this.data,
      read: read ?? this.read,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (msgId.present) {
      map['msg_id'] = Variable<BigInt>(msgId.value);
    }
    if (fromSession.present) {
      map['from_session'] = Variable<int>(fromSession.value);
    }
    if (sender.present) {
      map['sender'] = Variable<BigInt>(sender.value);
    }
    if (time.present) {
      map['time'] = Variable<DateTime>(time.value);
    }
    if (data.present) {
      map['data'] = Variable<String>(data.value);
    }
    if (read.present) {
      map['read'] = Variable<int>(read.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('RecordCompanion(')
          ..write('msgId: $msgId, ')
          ..write('fromSession: $fromSession, ')
          ..write('sender: $sender, ')
          ..write('time: $time, ')
          ..write('data: $data, ')
          ..write('read: $read')
          ..write(')'))
        .toString();
  }
}

abstract class _$OurchatDatabase extends GeneratedDatabase {
  _$OurchatDatabase(QueryExecutor e) : super(e);
  $OurchatDatabaseManager get managers => $OurchatDatabaseManager(this);
  late final $AccountTable account = $AccountTable(this);
  late final $SessionTable session = $SessionTable(this);
  late final $RecordTable record = $RecordTable(this);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities =>
      [account, session, record];
}

typedef $$AccountTableCreateCompanionBuilder = AccountCompanion Function({
  required BigInt id,
  required String email,
  required DateTime registerTime,
  required DateTime updateTime,
  required String friendsJson,
  required String sessionsJson,
  required DateTime latestMsgTime,
  Value<int> rowid,
});
typedef $$AccountTableUpdateCompanionBuilder = AccountCompanion Function({
  Value<BigInt> id,
  Value<String> email,
  Value<DateTime> registerTime,
  Value<DateTime> updateTime,
  Value<String> friendsJson,
  Value<String> sessionsJson,
  Value<DateTime> latestMsgTime,
  Value<int> rowid,
});

class $$AccountTableFilterComposer
    extends Composer<_$OurchatDatabase, $AccountTable> {
  $$AccountTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<BigInt> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get email => $composableBuilder(
      column: $table.email, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get registerTime => $composableBuilder(
      column: $table.registerTime, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get updateTime => $composableBuilder(
      column: $table.updateTime, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get friendsJson => $composableBuilder(
      column: $table.friendsJson, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get sessionsJson => $composableBuilder(
      column: $table.sessionsJson, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get latestMsgTime => $composableBuilder(
      column: $table.latestMsgTime, builder: (column) => ColumnFilters(column));
}

class $$AccountTableOrderingComposer
    extends Composer<_$OurchatDatabase, $AccountTable> {
  $$AccountTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<BigInt> get id => $composableBuilder(
      column: $table.id, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get email => $composableBuilder(
      column: $table.email, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get registerTime => $composableBuilder(
      column: $table.registerTime,
      builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get updateTime => $composableBuilder(
      column: $table.updateTime, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get friendsJson => $composableBuilder(
      column: $table.friendsJson, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get sessionsJson => $composableBuilder(
      column: $table.sessionsJson,
      builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get latestMsgTime => $composableBuilder(
      column: $table.latestMsgTime,
      builder: (column) => ColumnOrderings(column));
}

class $$AccountTableAnnotationComposer
    extends Composer<_$OurchatDatabase, $AccountTable> {
  $$AccountTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<BigInt> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get email =>
      $composableBuilder(column: $table.email, builder: (column) => column);

  GeneratedColumn<DateTime> get registerTime => $composableBuilder(
      column: $table.registerTime, builder: (column) => column);

  GeneratedColumn<DateTime> get updateTime => $composableBuilder(
      column: $table.updateTime, builder: (column) => column);

  GeneratedColumn<String> get friendsJson => $composableBuilder(
      column: $table.friendsJson, builder: (column) => column);

  GeneratedColumn<String> get sessionsJson => $composableBuilder(
      column: $table.sessionsJson, builder: (column) => column);

  GeneratedColumn<DateTime> get latestMsgTime => $composableBuilder(
      column: $table.latestMsgTime, builder: (column) => column);
}

class $$AccountTableTableManager extends RootTableManager<
    _$OurchatDatabase,
    $AccountTable,
    AccountData,
    $$AccountTableFilterComposer,
    $$AccountTableOrderingComposer,
    $$AccountTableAnnotationComposer,
    $$AccountTableCreateCompanionBuilder,
    $$AccountTableUpdateCompanionBuilder,
    (
      AccountData,
      BaseReferences<_$OurchatDatabase, $AccountTable, AccountData>
    ),
    AccountData,
    PrefetchHooks Function()> {
  $$AccountTableTableManager(_$OurchatDatabase db, $AccountTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$AccountTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$AccountTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$AccountTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<BigInt> id = const Value.absent(),
            Value<String> email = const Value.absent(),
            Value<DateTime> registerTime = const Value.absent(),
            Value<DateTime> updateTime = const Value.absent(),
            Value<String> friendsJson = const Value.absent(),
            Value<String> sessionsJson = const Value.absent(),
            Value<DateTime> latestMsgTime = const Value.absent(),
            Value<int> rowid = const Value.absent(),
          }) =>
              AccountCompanion(
            id: id,
            email: email,
            registerTime: registerTime,
            updateTime: updateTime,
            friendsJson: friendsJson,
            sessionsJson: sessionsJson,
            latestMsgTime: latestMsgTime,
            rowid: rowid,
          ),
          createCompanionCallback: ({
            required BigInt id,
            required String email,
            required DateTime registerTime,
            required DateTime updateTime,
            required String friendsJson,
            required String sessionsJson,
            required DateTime latestMsgTime,
            Value<int> rowid = const Value.absent(),
          }) =>
              AccountCompanion.insert(
            id: id,
            email: email,
            registerTime: registerTime,
            updateTime: updateTime,
            friendsJson: friendsJson,
            sessionsJson: sessionsJson,
            latestMsgTime: latestMsgTime,
            rowid: rowid,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$AccountTableProcessedTableManager = ProcessedTableManager<
    _$OurchatDatabase,
    $AccountTable,
    AccountData,
    $$AccountTableFilterComposer,
    $$AccountTableOrderingComposer,
    $$AccountTableAnnotationComposer,
    $$AccountTableCreateCompanionBuilder,
    $$AccountTableUpdateCompanionBuilder,
    (
      AccountData,
      BaseReferences<_$OurchatDatabase, $AccountTable, AccountData>
    ),
    AccountData,
    PrefetchHooks Function()>;
typedef $$SessionTableCreateCompanionBuilder = SessionCompanion Function({
  Value<BigInt> sessionId,
  required String members,
  required String roles,
});
typedef $$SessionTableUpdateCompanionBuilder = SessionCompanion Function({
  Value<BigInt> sessionId,
  Value<String> members,
  Value<String> roles,
});

class $$SessionTableFilterComposer
    extends Composer<_$OurchatDatabase, $SessionTable> {
  $$SessionTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<BigInt> get sessionId => $composableBuilder(
      column: $table.sessionId, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get members => $composableBuilder(
      column: $table.members, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get roles => $composableBuilder(
      column: $table.roles, builder: (column) => ColumnFilters(column));
}

class $$SessionTableOrderingComposer
    extends Composer<_$OurchatDatabase, $SessionTable> {
  $$SessionTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<BigInt> get sessionId => $composableBuilder(
      column: $table.sessionId, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get members => $composableBuilder(
      column: $table.members, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get roles => $composableBuilder(
      column: $table.roles, builder: (column) => ColumnOrderings(column));
}

class $$SessionTableAnnotationComposer
    extends Composer<_$OurchatDatabase, $SessionTable> {
  $$SessionTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<BigInt> get sessionId =>
      $composableBuilder(column: $table.sessionId, builder: (column) => column);

  GeneratedColumn<String> get members =>
      $composableBuilder(column: $table.members, builder: (column) => column);

  GeneratedColumn<String> get roles =>
      $composableBuilder(column: $table.roles, builder: (column) => column);
}

class $$SessionTableTableManager extends RootTableManager<
    _$OurchatDatabase,
    $SessionTable,
    SessionData,
    $$SessionTableFilterComposer,
    $$SessionTableOrderingComposer,
    $$SessionTableAnnotationComposer,
    $$SessionTableCreateCompanionBuilder,
    $$SessionTableUpdateCompanionBuilder,
    (
      SessionData,
      BaseReferences<_$OurchatDatabase, $SessionTable, SessionData>
    ),
    SessionData,
    PrefetchHooks Function()> {
  $$SessionTableTableManager(_$OurchatDatabase db, $SessionTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$SessionTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$SessionTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$SessionTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<BigInt> sessionId = const Value.absent(),
            Value<String> members = const Value.absent(),
            Value<String> roles = const Value.absent(),
          }) =>
              SessionCompanion(
            sessionId: sessionId,
            members: members,
            roles: roles,
          ),
          createCompanionCallback: ({
            Value<BigInt> sessionId = const Value.absent(),
            required String members,
            required String roles,
          }) =>
              SessionCompanion.insert(
            sessionId: sessionId,
            members: members,
            roles: roles,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$SessionTableProcessedTableManager = ProcessedTableManager<
    _$OurchatDatabase,
    $SessionTable,
    SessionData,
    $$SessionTableFilterComposer,
    $$SessionTableOrderingComposer,
    $$SessionTableAnnotationComposer,
    $$SessionTableCreateCompanionBuilder,
    $$SessionTableUpdateCompanionBuilder,
    (
      SessionData,
      BaseReferences<_$OurchatDatabase, $SessionTable, SessionData>
    ),
    SessionData,
    PrefetchHooks Function()>;
typedef $$RecordTableCreateCompanionBuilder = RecordCompanion Function({
  Value<BigInt> msgId,
  Value<int?> fromSession,
  required BigInt sender,
  required DateTime time,
  required String data,
  Value<int> read,
});
typedef $$RecordTableUpdateCompanionBuilder = RecordCompanion Function({
  Value<BigInt> msgId,
  Value<int?> fromSession,
  Value<BigInt> sender,
  Value<DateTime> time,
  Value<String> data,
  Value<int> read,
});

class $$RecordTableFilterComposer
    extends Composer<_$OurchatDatabase, $RecordTable> {
  $$RecordTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<BigInt> get msgId => $composableBuilder(
      column: $table.msgId, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get fromSession => $composableBuilder(
      column: $table.fromSession, builder: (column) => ColumnFilters(column));

  ColumnFilters<BigInt> get sender => $composableBuilder(
      column: $table.sender, builder: (column) => ColumnFilters(column));

  ColumnFilters<DateTime> get time => $composableBuilder(
      column: $table.time, builder: (column) => ColumnFilters(column));

  ColumnFilters<String> get data => $composableBuilder(
      column: $table.data, builder: (column) => ColumnFilters(column));

  ColumnFilters<int> get read => $composableBuilder(
      column: $table.read, builder: (column) => ColumnFilters(column));
}

class $$RecordTableOrderingComposer
    extends Composer<_$OurchatDatabase, $RecordTable> {
  $$RecordTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<BigInt> get msgId => $composableBuilder(
      column: $table.msgId, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get fromSession => $composableBuilder(
      column: $table.fromSession, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<BigInt> get sender => $composableBuilder(
      column: $table.sender, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<DateTime> get time => $composableBuilder(
      column: $table.time, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<String> get data => $composableBuilder(
      column: $table.data, builder: (column) => ColumnOrderings(column));

  ColumnOrderings<int> get read => $composableBuilder(
      column: $table.read, builder: (column) => ColumnOrderings(column));
}

class $$RecordTableAnnotationComposer
    extends Composer<_$OurchatDatabase, $RecordTable> {
  $$RecordTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<BigInt> get msgId =>
      $composableBuilder(column: $table.msgId, builder: (column) => column);

  GeneratedColumn<int> get fromSession => $composableBuilder(
      column: $table.fromSession, builder: (column) => column);

  GeneratedColumn<BigInt> get sender =>
      $composableBuilder(column: $table.sender, builder: (column) => column);

  GeneratedColumn<DateTime> get time =>
      $composableBuilder(column: $table.time, builder: (column) => column);

  GeneratedColumn<String> get data =>
      $composableBuilder(column: $table.data, builder: (column) => column);

  GeneratedColumn<int> get read =>
      $composableBuilder(column: $table.read, builder: (column) => column);
}

class $$RecordTableTableManager extends RootTableManager<
    _$OurchatDatabase,
    $RecordTable,
    RecordData,
    $$RecordTableFilterComposer,
    $$RecordTableOrderingComposer,
    $$RecordTableAnnotationComposer,
    $$RecordTableCreateCompanionBuilder,
    $$RecordTableUpdateCompanionBuilder,
    (RecordData, BaseReferences<_$OurchatDatabase, $RecordTable, RecordData>),
    RecordData,
    PrefetchHooks Function()> {
  $$RecordTableTableManager(_$OurchatDatabase db, $RecordTable table)
      : super(TableManagerState(
          db: db,
          table: table,
          createFilteringComposer: () =>
              $$RecordTableFilterComposer($db: db, $table: table),
          createOrderingComposer: () =>
              $$RecordTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer: () =>
              $$RecordTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback: ({
            Value<BigInt> msgId = const Value.absent(),
            Value<int?> fromSession = const Value.absent(),
            Value<BigInt> sender = const Value.absent(),
            Value<DateTime> time = const Value.absent(),
            Value<String> data = const Value.absent(),
            Value<int> read = const Value.absent(),
          }) =>
              RecordCompanion(
            msgId: msgId,
            fromSession: fromSession,
            sender: sender,
            time: time,
            data: data,
            read: read,
          ),
          createCompanionCallback: ({
            Value<BigInt> msgId = const Value.absent(),
            Value<int?> fromSession = const Value.absent(),
            required BigInt sender,
            required DateTime time,
            required String data,
            Value<int> read = const Value.absent(),
          }) =>
              RecordCompanion.insert(
            msgId: msgId,
            fromSession: fromSession,
            sender: sender,
            time: time,
            data: data,
            read: read,
          ),
          withReferenceMapper: (p0) => p0
              .map((e) => (e.readTable(table), BaseReferences(db, table, e)))
              .toList(),
          prefetchHooksCallback: null,
        ));
}

typedef $$RecordTableProcessedTableManager = ProcessedTableManager<
    _$OurchatDatabase,
    $RecordTable,
    RecordData,
    $$RecordTableFilterComposer,
    $$RecordTableOrderingComposer,
    $$RecordTableAnnotationComposer,
    $$RecordTableCreateCompanionBuilder,
    $$RecordTableUpdateCompanionBuilder,
    (RecordData, BaseReferences<_$OurchatDatabase, $RecordTable, RecordData>),
    RecordData,
    PrefetchHooks Function()>;

class $OurchatDatabaseManager {
  final _$OurchatDatabase _db;
  $OurchatDatabaseManager(this._db);
  $$AccountTableTableManager get account =>
      $$AccountTableTableManager(_db, _db.account);
  $$SessionTableTableManager get session =>
      $$SessionTableTableManager(_db, _db.session);
  $$RecordTableTableManager get record =>
      $$RecordTableTableManager(_db, _db.record);
}
