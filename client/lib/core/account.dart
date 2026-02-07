import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:grpc/grpc.dart';
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

  Future<bool> login(String password, String? ocid, String? email) async {
    AuthServiceClient authStub = AuthServiceClient(server.channel!);
    var l10n = ourchatAppState.l10n;
    try {
      logger.d("login");
      var res = await safeRequest(
          authStub.auth,
          AuthRequest(
            email: email,
            ocid: ocid,
            password: password,
          ), (GrpcError e) {
        showResultMessage(ourchatAppState, e.code, e.message,
            notFoundStatus: l10n.notFound(l10n.user),
            invalidArgumentStatus: l10n.internalError,
            unauthenticatedStatus: l10n.incorrectPassword);
      }, rethrowError: true);

      email = email;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurChatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> register(String password, String name, String email) async {
    AuthServiceClient authStub = AuthServiceClient(server.channel!);
    var l10n = ourchatAppState.l10n;
    try {
      logger.d("register");
      var res = await safeRequest(
          authStub.register,
          RegisterRequest(
            email: email,
            password: password,
            name: name,
            // temporary hardcoded RSA 4096 public key
            // When using e2ee, this key should be generated on client side
            publicKey: """-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4HIApvn1bfFYdUQCfvyc
dJwVRVs0MNwoKN4gRSnNm3jsOF3WOG2boXKwnsMG0BPAOvT5/UHhme7xsD0K7DzF
tXjL0d1ntBQcPddcH7nufnyfYxHJp7p5VfXc0T3xUB4Sn4bprIZ/zS1e0u64TYUJ
03hfiSXgzsrSt7JCI6QEMfP8mdIAItIbzS3I/RwnmdvreyMdzzjajcPvVOHKq5NN
7VUtfpiZbEhzJECHkgPVpzQe4cuQIwNGtPqhEb+uRe0Lsolpvw5wfihx1nx6j3X9
aoOj+FKc4agHnVwZavuV9s0T6Pg9017iplMbzXeZEWo0hwQa0rhFNvB90beAyCjp
6wIDAQAB
-----END PUBLIC KEY-----"""
                .codeUnits,
          ), (GrpcError e) {
        logger.w("register fail: code ${e.code}, message: ${e.message}");
        // 处理报错
        showResultMessage(ourchatAppState, e.code, e.message,
            alreadyExistsStatus: l10n.alreadyExists(l10n.email),
            invalidArgumentStatus: {
              "Password Is Not Strong Enough": l10n.passwordIsNotStrongEnough,
              "Username Is Invalid": l10n.invalid(l10n.username),
              "Email Address Is Invalid": l10n.invalid(l10n.email),
            });
      }, rethrowError: true);
      email = email;
      username = name;
      id = res.id;
      ocid = res.ocid;
      isMe = true;
      var interceptor = OurChatInterceptor();
      interceptor.setToken(res.token);
      server.interceptor = interceptor;
      recreateStub();
      return true;
    } catch (e) {
      return false;
    }
  }

  Future getAccountInfo({bool ignoreCache = false}) async {
    logger.d("get account info for id: ${id.toString()}");
    if (ourchatAppState.gettingInfoAccountList.contains(id)) {
      while (ourchatAppState.gettingInfoAccountList.contains(id)) {
        await Future.delayed(Duration(milliseconds: 10));
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
        logger.d("use account info cache");
        return true;
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
      try {
        GetAccountInfoResponse res = await safeRequest(
            stub.getAccountInfo,
            GetAccountInfoRequest(
                id: id,
                requestValues: [QueryValues.QUERY_VALUES_PUBLIC_UPDATED_TIME]),
            getAccountInfoOnError,
            rethrowError: true);
        if (OurChatTime(inputTimestamp: res.publicUpdatedTime) !=
            OurChatTime(inputDatetime: publicData.publicUpdateTime)) {
          publicDataNeedUpdate = true;
        }
      } catch (e) {
        ourchatAppState.gettingInfoAccountList.remove(id);
        return false;
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
      try {
        GetAccountInfoResponse res = await safeRequest(
            stub.getAccountInfo,
            GetAccountInfoRequest(
                id: id, requestValues: [QueryValues.QUERY_VALUES_UPDATED_TIME]),
            getAccountInfoOnError,
            rethrowError: true);
        if (OurChatTime(inputTimestamp: res.updatedTime) !=
            OurChatTime(inputDatetime: privateData.updateTime)) {
          privateDataNeedUpdate = true;
        }
      } catch (e) {
        ourchatAppState.gettingInfoAccountList.remove(id);
        return false;
      }
    }
    logger.d(
        "accountId: $id,isMe: $isMe, private data need update: $privateDataNeedUpdate, public data need update: $publicDataNeedUpdate");
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
      try {
        GetAccountInfoResponse res = await safeRequest(
            stub.getAccountInfo,
            GetAccountInfoRequest(id: id, requestValues: requestValues),
            getAccountInfoOnError,
            rethrowError: true);
        if (publicDataNeedUpdate) {
          await updatePublicData(res, publicData != null);
        }
        if (privateDataNeedUpdate) {
          await updatePrivateData(res, privateData != null);
        }
        if (ourchatAppState.thisAccount!.friends.contains(id)) {
          // get displayname
          logger.d("get account display_name info");
          res = await safeRequest(
              stub.getAccountInfo,
              GetAccountInfoRequest(
                  id: id,
                  requestValues: [QueryValues.QUERY_VALUES_DISPLAY_NAME]),
              getAccountInfoOnError,
              rethrowError: true);
          displayName = res.displayName;
        }
      } catch (e) {
        ourchatAppState.gettingInfoAccountList.remove(id);
        return false;
      }
    }
    lastCheckTime = DateTime.now();
    ourchatAppState.accountCachePool[id] = this;
    ourchatAppState.gettingInfoAccountList.remove(id);
    logger.d("save account info to cache");
    return true;
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

  void getAccountInfoOnError(GrpcError e) {
    var l10n = ourchatAppState.l10n;
    showResultMessage(
      ourchatAppState,
      e.code,
      e.message,
      notFoundStatus: l10n.notFound(l10n.user),
      permissionDeniedStatus: l10n.permissionDenied("Get Account Info"),
      invalidArgumentStatus: l10n.internalError,
    );
  }
}
