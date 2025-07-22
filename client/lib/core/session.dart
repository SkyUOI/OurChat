import 'dart:convert';

import 'package:drift/drift.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/session/get_session_info/v1/get_session_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';

class OurchatSession {
  OurchatAppState ourchatAppState;
  late OurchatServer server;
  late OurChatServiceClient stub;
  Int64 sessionId;
  late String name;
  late String? avatarKey;
  late OurchatTime createdTime, updatedTime;
  late List<Int64> members = [];
  late Map<Int64, int> roles = {};
  late int size;

  OurchatSession(this.ourchatAppState, this.sessionId) {
    server = ourchatAppState.server!;
    stub = OurChatServiceClient(server.channel!,
        interceptors: [server.interceptor!]);
  }

  Future getSessionInfo() async {
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
      publicNeedUpdate = (OurchatTime(inputTimestamp: res.updatedTime) ==
          OurchatTime(inputDatetime: localSessionData.updatedTime));
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
        QueryValues.QUERY_VALUES_SIZE,
        QueryValues.QUERY_VALUES_DESCRIPTION,
      ]));
      name = res.name;
      avatarKey = res.avatarKey;
      createdTime = OurchatTime(inputTimestamp: res.createdTime);
      updatedTime = OurchatTime(inputTimestamp: res.updatedTime);
      size = res.size.toInt();

      if (localSessionData == null) {
        var publicDB = ourchatAppState.publicDB;
        publicDB.into(publicDB.publicSession).insert(PublicSessionData(
            sessionId: BigInt.from(sessionId.toInt()),
            name: res.name,
            createdTime: createdTime.datetime,
            updatedTime: updatedTime.datetime,
            size: size));
      }
    } else {
      name = localSessionData!.name;
      avatarKey = localSessionData.avatarKey;
      createdTime = OurchatTime(inputDatetime: localSessionData.createdTime);
      updatedTime = OurchatTime(inputDatetime: localSessionData.updatedTime);
      size = localSessionData.size;
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
  }
}
