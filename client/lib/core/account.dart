import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/google/protobuf/timestamp.pb.dart';
import 'package:ourchat/service/auth/authorize/v1/authorize.pb.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:ourchat/service/ourchat/get_account_info/v1/get_account_info.pb.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc.dart';
import 'package:crypto/crypto.dart';

class OurchatAccount {
  OurchatAppState ourchatAppState;
  late OurchatServer server;
  late Int64 id;
  late String username, email, ocid;
  String? avatarKey, displayName, status;
  bool isMe = false;
  late OurchatTime publicUpdateTime, updatedTime, registerTime;
  late List<Int64> friends, sessions;
  late OurChatServiceClient stub;

  // 客户端独有字段，仅isMe为True时使用
  OurchatTime latestMsgTime = OurchatTime(inputTimestamp: Timestamp());

  OurchatAccount(this.ourchatAppState) {
    server = ourchatAppState.server!;
    stub = OurChatServiceClient(server.channel!);
  }

  void recreateStub() {
    stub = OurChatServiceClient(server.channel!,
        interceptors: [server.interceptor!]);
  }

  Future login(String password, String? ocid, String? email) async {
    AuthServiceClient authStub = AuthServiceClient(server.channel!);
    try {
      var res = await authStub.auth(
        AuthRequest(
          email: email,
          ocid: ocid,
          password: sha256.convert(ascii.encode(password)).toString(),
        ),
      );
      email = email;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurchatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return okStatusCode;
    } on GrpcError catch (e) {
      return e.code;
    }
  }

  Future register(String password, String name, String email) async {
    AuthServiceClient authStub = AuthServiceClient(server.channel!);

    try {
      var res = await authStub.register(
        RegisterRequest(
          email: email,
          password: sha256.convert(ascii.encode(password)).toString(),
          name: name,
        ),
      );
      email = email;
      username = name;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurchatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return okStatusCode;
    } on GrpcError catch (e) {
      return e.code;
    }
  }

  Future getAccountInfo() async {
    PublicOurchatDatabase db = ourchatAppState.publicDB;
    OurchatDatabase pdb = ourchatAppState.privateDB!;
    bool publicDataNeedUpdate = false, privateDataNeedUpdate = false;
    var publicData = await (db.select(db.publicAccount)
          ..where((u) => u.id.equals(BigInt.from((id.toInt())))))
        .getSingleOrNull();
    var privateData = await (pdb.select(pdb.account)
          ..where((u) => u.id.equals(BigInt.from((id.toInt())))))
        .getSingleOrNull();
    if (publicData == null) {
      publicDataNeedUpdate = true;
    } else {
      GetAccountInfoResponse res = await stub.getAccountInfo(
          GetAccountInfoRequest(
              id: id,
              requestValues: [QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME]));
      if (OurchatTime(inputTimestamp: res.publicUpdatedTime) !=
          OurchatTime(inputDatetime: publicData.publicUpdateTime)) {
        publicDataNeedUpdate = true;
      }
    }
    if (privateData == null) {
      if (isMe) {
        privateDataNeedUpdate = true;
      }
    } else {
      GetAccountInfoResponse res = await stub.getAccountInfo(
          GetAccountInfoRequest(
              id: id, requestValues: [QueryValues.QUERY_VALUES_UPDATED_TIME]));
      if (OurchatTime(inputTimestamp: res.updatedTime) !=
          OurchatTime(inputDatetime: privateData.updateTime)) {
        privateDataNeedUpdate = true;
      }
    }

    if (publicDataNeedUpdate) await updatePublicData(publicData != null);
    if (privateDataNeedUpdate) {
      await updatePrivateData(privateData != null);
    }

    if (ourchatAppState.thisAccount!.friends.contains(id)) {
      // get displayname
      var res = await stub.getAccountInfo(GetAccountInfoRequest(
          id: id, requestValues: [QueryValues.QUERY_VALUES_DISPLAY_NAME]));
      displayName = res.displayName;
    }
  }

  Future updatePublicData(bool isDataExist) async {
    GetAccountInfoResponse res =
        await stub.getAccountInfo(GetAccountInfoRequest(id: id, requestValues: [
      QueryValues.QUERY_VALUES_AVATAR_KEY,
      QueryValues.QUERY_VALUES_USER_NAME,
      QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME,
      QueryValues.QUERY_VALUES_STATUS,
      QueryValues.QUERY_VALUES_OCID
    ]));
    avatarKey = res.avatarKey;
    username = res.userName;
    publicUpdateTime = OurchatTime(inputTimestamp: res.publicUpdatedTime);
    status = res.status;
    ocid = res.ocid;
    PublicOurchatDatabase publicDB = ourchatAppState.publicDB;
    if (isDataExist) {
      // 更新数据
      (publicDB.update(publicDB.publicAccount)
            ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
          .write(PublicAccountCompanion(
              avatarKey: Value(avatarKey),
              username: Value(username),
              publicUpdateTime: Value(publicUpdateTime.datetime),
              status: Value(status),
              ocid: Value(ocid)));
    } else {
      publicDB.into(publicDB.publicAccount).insert(PublicAccountData(
            id: BigInt.from(id.toInt()),
            avatarKey: avatarKey,
            username: username,
            publicUpdateTime: publicUpdateTime.datetime,
            status: status,
            ocid: ocid,
          ));
    }
  }

  Future updatePrivateData(isDataExist) async {
    var res = await stub.getAccountInfo(
      GetAccountInfoRequest(
        id: id,
        requestValues: [
          QueryValues.QUERY_VALUES_UPDATED_TIME,
          QueryValues.QUERY_VALUES_SESSIONS,
          QueryValues.QUERY_VALUES_FRIENDS,
          QueryValues.QUERY_VALUES_EMAIL,
          QueryValues.QUERY_VALUES_REGISTER_TIME,
        ],
      ),
    );
    updatedTime = OurchatTime(inputTimestamp: res.updatedTime);
    email = res.email;
    friends = res.friends;
    sessions = res.sessions;
    registerTime = OurchatTime(inputTimestamp: res.registerTime);
    OurchatDatabase? privateDB = ourchatAppState.privateDB;
    var intFriendsId = [];
    var intSessionsId = [];
    for (int i = 0; i < friends.length; i++) {
      intFriendsId.add(friends[i].toInt());
    }
    for (int i = 0; i < sessions.length; i++) {
      intSessionsId.add(sessions[i].toInt());
    }
    if (isDataExist) {
      (privateDB!.update(privateDB.account)
            ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
          .write(AccountCompanion(
              email: Value(email),
              registerTime: Value(registerTime.datetime),
              updateTime: Value(updatedTime.datetime),
              friendsJson: Value(jsonEncode(intFriendsId)),
              sessionsJson: Value(jsonEncode(intSessionsId)),
              latestMsgTime: Value(latestMsgTime.datetime)));
    } else {
      privateDB!.into(privateDB.account).insert(AccountData(
          id: BigInt.from(id.toInt()),
          email: email,
          registerTime: registerTime.datetime,
          updateTime: updatedTime.datetime,
          friendsJson: jsonEncode(intFriendsId),
          sessionsJson: jsonEncode(intSessionsId),
          latestMsgTime: latestMsgTime.datetime));
    }
  }

  void updateLatestMsgTime() {
    var pdb = ourchatAppState.privateDB!;
    (pdb.update(pdb.account)
          ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
        .write(AccountCompanion(latestMsgTime: Value(latestMsgTime.datetime)));
  }
}
