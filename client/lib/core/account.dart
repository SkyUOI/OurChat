import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/log.dart';
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

class OurChatAccount {
  OurChatAppState ourchatAppState;
  late OurChatServer server;
  late Int64 id;
  late String username, ocid;
  String? avatarKey, displayName, status, email;
  bool isMe = false;
  late OurChatTime publicUpdateTime, updatedTime, registerTime;
  DateTime lastCheckTime = DateTime(0);
  late List<Int64> friends, sessions;
  late OurChatServiceClient stub;

  // 客户端独有字段，仅isMe为True时使用
  OurChatTime latestMsgTime = OurChatTime(inputTimestamp: Timestamp());

  OurChatAccount(this.ourchatAppState) {
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
      logger.d("login");
      var res = await authStub.auth(
        AuthRequest(email: email, ocid: ocid, password: password),
      );
      email = email;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurChatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return (okStatusCode, null);
    } on GrpcError catch (e) {
      return (e.code, e.message);
    }
  }

  Future register(String password, String name, String email) async {
    AuthServiceClient authStub = AuthServiceClient(server.channel!);
    try {
      logger.d("register");
      var res = await authStub.register(
        RegisterRequest(
          email: email,
          password: password,
          name: name,
        ),
      );
      email = email;
      username = name;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurChatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return (okStatusCode, null);
    } on GrpcError catch (e) {
      return (e.code, e.message);
    }
  }

  Future getAccountInfo({bool ignoreCache = false}) async {
    logger.d("get account info for id: ${id.toString()}");
    if (ourchatAppState.gettingInfoAccountList.contains(id)) {
      while (ourchatAppState.gettingInfoAccountList.contains(id)) {
        await Future.delayed(Duration(milliseconds: 100));
      }
      await getAccountInfo(ignoreCache: ignoreCache);
      return;
    }
    ourchatAppState.gettingInfoAccountList.add(id);
    if (ourchatAppState.accountCachePool.containsKey(id)) {
      OurChatAccount accountCache = ourchatAppState.accountCachePool[id]!;
      if (!ignoreCache &&
          DateTime.now().difference(accountCache.lastCheckTime).inMinutes < 5) {
        // 上次检查更新在5min内 无需检查
        username = accountCache.username;
        ocid = accountCache.ocid;
        displayName = accountCache.displayName;
        status = accountCache.status;
        isMe = accountCache.isMe;
        publicUpdateTime = accountCache.publicUpdateTime;
        lastCheckTime = accountCache.lastCheckTime;
        stub = accountCache.stub;
        if (isMe) {
          email = accountCache.email;
          updatedTime = accountCache.updatedTime;
          registerTime = accountCache.registerTime;
          friends = accountCache.friends;
          sessions = accountCache.sessions;
        }
        ourchatAppState.gettingInfoAccountList.remove(id);
        return;
      }
    }

    List<QueryValues> requestValues = [];
    PublicOurChatDatabase db = ourchatAppState.publicDB;
    OurChatDatabase pdb = ourchatAppState.privateDB!;
    if (ourchatAppState.thisAccount != null &&
        ourchatAppState.thisAccount!.id == id) {
      isMe = true;
    }
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
      logger.d("get account public updated time");
      GetAccountInfoResponse res = await stub.getAccountInfo(
          GetAccountInfoRequest(
              id: id,
              requestValues: [QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME]));
      if (OurChatTime(inputTimestamp: res.publicUpdatedTime) !=
          OurChatTime(inputDatetime: publicData.publicUpdateTime)) {
        publicDataNeedUpdate = true;
      }
    }
    if (!publicDataNeedUpdate) {
      // 使用本地缓存
      username = publicData!.username;
      ocid = publicData.ocid;
      avatarKey = publicData.avatarKey;
      status = publicData.status;
      publicUpdateTime =
          OurChatTime(inputDatetime: publicData.publicUpdateTime);
    }
    if (privateData == null) {
      if (isMe) {
        privateDataNeedUpdate = true;
      }
    } else {
      logger.d("get account private updated time");
      GetAccountInfoResponse res = await stub.getAccountInfo(
          GetAccountInfoRequest(
              id: id, requestValues: [QueryValues.QUERY_VALUES_UPDATED_TIME]));
      if (OurChatTime(inputTimestamp: res.updatedTime) !=
          OurChatTime(inputDatetime: privateData.updateTime)) {
        privateDataNeedUpdate = true;
      }
    }
    if (!privateDataNeedUpdate && isMe) {
      // 使用本地缓存
      updatedTime = OurChatTime(inputDatetime: privateData!.updateTime);
      email = privateData.email;
      friends = [];
      List<dynamic> friendsList = jsonDecode(privateData.friendsJson);
      for (int i = 0; i < friendsList.length; i++) {
        friends.add(Int64.parseInt(friendsList[i].toString()));
      }
      sessions = [];
      List<dynamic> sessionsList = jsonDecode(privateData.sessionsJson);
      for (int i = 0; i < sessionsList.length; i++) {
        sessions.add(Int64.parseInt(sessionsList[i].toString()));
      }
      registerTime = OurChatTime(inputDatetime: privateData.registerTime);
    }

    if (publicDataNeedUpdate) {
      requestValues.addAll([
        QueryValues.QUERY_VALUES_AVATAR_KEY,
        QueryValues.QUERY_VALUES_USER_NAME,
        QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME,
        QueryValues.QUERY_VALUES_STATUS,
        QueryValues.QUERY_VALUES_OCID
      ]);
    }
    if (privateDataNeedUpdate) {
      requestValues.addAll([
        QueryValues.QUERY_VALUES_UPDATED_TIME,
        QueryValues.QUERY_VALUES_SESSIONS,
        QueryValues.QUERY_VALUES_FRIENDS,
        QueryValues.QUERY_VALUES_EMAIL,
        QueryValues.QUERY_VALUES_REGISTER_TIME,
      ]);
    }
    if (publicDataNeedUpdate ||
        privateDataNeedUpdate ||
        ourchatAppState.thisAccount!.friends.contains(id)) {
      logger.d("get account info");
      GetAccountInfoResponse res = await stub.getAccountInfo(
          GetAccountInfoRequest(id: id, requestValues: requestValues));
      if (publicDataNeedUpdate) {
        await updatePublicData(res, publicData != null);
      }
      if (privateDataNeedUpdate) {
        await updatePrivateData(res, privateData != null);
      }
      if (ourchatAppState.thisAccount!.friends.contains(id)) {
        // get displayname
        logger.d("get account display_name info");
        res = await stub.getAccountInfo(GetAccountInfoRequest(
            id: id, requestValues: [QueryValues.QUERY_VALUES_DISPLAY_NAME]));
        displayName = res.displayName;
      }
    }

    lastCheckTime = DateTime.now();
    ourchatAppState.accountCachePool[id] = this;
    ourchatAppState.gettingInfoAccountList.remove(id);
    logger.d("save account info to cache");
  }

  Future updatePublicData(GetAccountInfoResponse res, bool isDataExist) async {
    logger.d("get account public info");
    avatarKey = res.avatarKey;
    username = res.userName;
    publicUpdateTime = OurChatTime(inputTimestamp: res.publicUpdatedTime);
    status = res.status;
    ocid = res.ocid;
    PublicOurChatDatabase publicDB = ourchatAppState.publicDB;
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

  Future updatePrivateData(GetAccountInfoResponse res, bool isDataExist) async {
    logger.d("update account private info");
    updatedTime = OurChatTime(inputTimestamp: res.updatedTime);
    email = res.email;
    friends = res.friends;
    sessions = res.sessions;
    registerTime = OurChatTime(inputTimestamp: res.registerTime);
    OurChatDatabase? privateDB = ourchatAppState.privateDB;
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
              email: Value(email!),
              registerTime: Value(registerTime.datetime),
              updateTime: Value(updatedTime.datetime),
              friendsJson: Value(jsonEncode(intFriendsId)),
              sessionsJson: Value(jsonEncode(intSessionsId)),
              latestMsgTime: Value(latestMsgTime.datetime)));
    } else {
      privateDB!.into(privateDB.account).insert(AccountData(
          id: BigInt.from(id.toInt()),
          email: email!,
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

  String avatarUrl() {
    return "http${ourchatAppState.server!.isTLS! ? 's' : ''}://${ourchatAppState.server!.host}:${ourchatAppState.server!.port.toString()}/v1/avatar?user_id=${id.toString()}";
  }

  String getNameWithDisplayName() {
    if (displayName != null && displayName != "" && displayName != username) {
      return "$displayName ($username)";
    }
    return username;
  }

  String getDisplayNameOrName() {
    if (displayName != null && displayName != "") {
      return displayName!;
    }
    return username;
  }

  @override
  bool operator ==(other) {
    return (other is OurChatAccount) && other.id == id;
  }

  @override
  int get hashCode => id.toInt();
}
