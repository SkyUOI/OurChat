import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/session/get_session_info/v1/get_session_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:ourchat/l10n/app_localizations.dart';

class OurChatSession {
  OurChatAppState ourchatAppState;
  late OurChatServer server;
  late OurChatServiceClient stub;
  Int64 sessionId;
  late String name, description;
  late String? avatarKey;
  late OurChatTime createdTime, updatedTime;
  late List<Int64> members = [];
  late Map<Int64, int> roles = {};
  late int size;
  String? displayName;
  DateTime lastCheckTime = DateTime(0);

  OurChatSession(this.ourchatAppState, this.sessionId) {
    server = ourchatAppState.server!;
    stub = OurChatServiceClient(server.channel!,
        interceptors: [server.interceptor!]);
  }

  Future getSessionInfo({bool ignoreCache = false}) async {
    if (ourchatAppState.sessionCachePool.keys.contains(sessionId)) {
      OurChatSession sessionCache =
          ourchatAppState.sessionCachePool[sessionId]!;
      if (!ignoreCache &&
          DateTime.now().difference(sessionCache.lastCheckTime).inMinutes < 5) {
        // 上次检查更新在5min内 无需检查
        sessionId = sessionCache.sessionId;
        name = sessionCache.name;
        avatarKey = sessionCache.avatarKey;
        createdTime = sessionCache.createdTime;
        updatedTime = sessionCache.updatedTime;
        members = sessionCache.members;
        roles = sessionCache.roles;
        size = sessionCache.size;
        description = sessionCache.description;
        lastCheckTime = sessionCache.lastCheckTime;
        displayName = sessionCache.displayName;
        return;
      }
    }

    var localSessionData = await (ourchatAppState.publicDB
            .select(ourchatAppState.publicDB.publicSession)
          ..where((u) => u.sessionId.equals(BigInt.from(sessionId.toInt()))))
        .getSingleOrNull();
    bool publicNeedUpdate = false, privateNeedUpdate = false;
    if (localSessionData == null) {
      publicNeedUpdate = true;
      privateNeedUpdate =
          ourchatAppState.thisAccount!.sessions.contains(sessionId) &&
              publicNeedUpdate;
    } else {
      GetSessionInfoResponse res = await stub.getSessionInfo(
          GetSessionInfoRequest(sessionId: sessionId, queryValues: [
        QueryValues.QUERY_VALUES_UPDATED_TIME,
      ]));
      publicNeedUpdate = (OurChatTime(inputTimestamp: res.updatedTime) !=
          OurChatTime(inputDatetime: localSessionData.updatedTime));
      privateNeedUpdate =
          ourchatAppState.thisAccount!.sessions.contains(sessionId) &&
              publicNeedUpdate;
    }

    if (publicNeedUpdate) {
      GetSessionInfoResponse res = await stub.getSessionInfo(
          GetSessionInfoRequest(sessionId: sessionId, queryValues: [
        QueryValues.QUERY_VALUES_NAME,
        QueryValues.QUERY_VALUES_AVATAR_KEY,
        QueryValues.QUERY_VALUES_CREATED_TIME,
        QueryValues.QUERY_VALUES_UPDATED_TIME,
        QueryValues.QUERY_VALUES_SIZE,
        QueryValues.QUERY_VALUES_DESCRIPTION,
      ]));
      name = res.name;
      avatarKey = res.avatarKey;
      createdTime = OurChatTime(inputTimestamp: res.createdTime);
      updatedTime = OurChatTime(inputTimestamp: res.updatedTime);
      size = res.size.toInt();
      description = res.description;

      if (localSessionData == null) {
        var publicDB = ourchatAppState.publicDB;
        publicDB.into(publicDB.publicSession).insert(PublicSessionData(
            sessionId: BigInt.from(sessionId.toInt()),
            name: res.name,
            createdTime: createdTime.datetime,
            updatedTime: updatedTime.datetime,
            size: size,
            description: description));
      } else {
        var publicDB = ourchatAppState.publicDB;
        (publicDB.update(publicDB.publicSession)
              ..where((u) => u.sessionId
                  .equals(BigInt.from(int.parse(sessionId.toString())))))
            .write(PublicSessionCompanion(
                name: Value(name),
                avatarKey: Value(avatarKey),
                createdTime: Value(createdTime.datetime),
                updatedTime: Value(updatedTime.datetime),
                size: Value(size),
                description: Value(description)));
      }
    } else {
      name = localSessionData!.name;
      avatarKey = localSessionData.avatarKey;
      createdTime = OurChatTime(inputDatetime: localSessionData.createdTime);
      updatedTime = OurChatTime(inputDatetime: localSessionData.updatedTime);
      size = localSessionData.size;
      description = localSessionData.description;
    }
    var privateDB = ourchatAppState.privateDB!;
    var localSessionPrivateData = await (privateDB.select(privateDB.session)
          ..where((u) => u.sessionId.equals(BigInt.from(sessionId.toInt()))))
        .getSingleOrNull();
    if (localSessionPrivateData == null) {
      privateNeedUpdate = true;
    }
    if (privateNeedUpdate) {
      GetSessionInfoResponse res = await stub.getSessionInfo(
          GetSessionInfoRequest(sessionId: sessionId, queryValues: [
        QueryValues.QUERY_VALUES_MEMBERS,
        QueryValues.QUERY_VALUES_ROLES,
      ]));
      members = res.members;
      for (int i = 0; i < res.roles.length; i++) {
        roles[res.roles[i].userId] = res.roles[i].role.toInt();
      }
      var intMembers = [];
      for (int i = 0; i < members.length; i++) {
        intMembers.add(members[i].toInt());
      }
      var jsonRoles = {};
      roles.forEach((key, value) => jsonRoles[key.toString()] = value);
      if (localSessionPrivateData == null) {
        privateDB.into(privateDB.session).insert(SessionData(
            sessionId: BigInt.from(sessionId.toInt()),
            members: jsonEncode(intMembers),
            roles: jsonEncode(jsonRoles)));
      } else {
        (privateDB.update(privateDB.session)
              ..where(
                  (u) => u.sessionId.equals(BigInt.from(sessionId.toInt()))))
            .write(SessionCompanion(
                members: Value(jsonEncode(intMembers)),
                roles: Value(jsonEncode(jsonRoles))));
      }
    } else if (ourchatAppState.thisAccount!.sessions.contains(sessionId)) {
      var privateDB = ourchatAppState.privateDB!;
      var localSessionPrivateData = await (privateDB.select(privateDB.session)
            ..where((u) => u.sessionId.equals(BigInt.from(sessionId.toInt()))))
          .getSingle();
      var intMembers = jsonDecode(localSessionPrivateData.members);
      var intRoles = jsonDecode(localSessionPrivateData.roles);
      for (int i = 0; i < intMembers.length; i++) {
        members.add(Int64.parseInt(intMembers[i].toString()));
      }
      intRoles.forEach((key, value) => roles[Int64.parseInt(key)] = value);
    }

    if (members.length == 2) {
      Int64 otherUserId = members
          .firstWhere((element) => element != ourchatAppState.thisAccount!.id);
      OurChatAccount otherAccount = OurChatAccount(ourchatAppState);
      otherAccount.id = otherUserId;
      otherAccount.recreateStub();
      await otherAccount.getAccountInfo();
      displayName = otherAccount.displayName;
    }
    lastCheckTime = DateTime.now();
    ourchatAppState.sessionCachePool[sessionId] = this;
  }

  @override
  int get hashCode => sessionId.toInt();

  @override
  bool operator ==(Object other) {
    if (other is OurChatSession) {
      return sessionId == other.sessionId;
    }
    return false;
  }

  String getDisplayName(AppLocalizations l10n) {
    if (name.isNotEmpty) {
      return name;
    }
    if (displayName == null) {
      return l10n.newSession;
    }
    return displayName!;
  }
}
