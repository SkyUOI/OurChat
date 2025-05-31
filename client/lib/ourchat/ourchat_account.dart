import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/ourchat/ourchat_chore.dart';
import 'package:ourchat/ourchat/ourchat_database.dart';
import 'package:ourchat/ourchat/ourchat_server.dart';
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
  late OurChatServer server;
  late Int64 id;
  late String username, avatarKey, displayName, status, email, ocid, token;
  bool isMe = false, gotInfo = false;
  late OurchatTime publicUpdateTime, updateTime, registerTime;
  late List<Int64> friends, sessions;
  late OurChatServiceClient stub;

  // 客户端独有字段，仅isMe为True时使用
  OurchatTime latestMsgTime = OurchatTime(inputTimestamp: Timestamp());

  OurchatAccount(this.ourchatAppState) {
    server = ourchatAppState.server!;
    stub = OurChatServiceClient(server.channel!);
  }

  void recreateStub() {
    var interceptor = OurchatInterceptor();
    interceptor.setToken(token);
    server.interceptor = interceptor;
    stub = OurChatServiceClient(server.channel!, interceptors: [interceptor]);
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
      token = res.token;
      isMe = true;
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
      token = res.token;
      isMe = true;
      recreateStub();
      return okStatusCode;
    } on GrpcError catch (e) {
      return e.code;
    }
  }

  Future getAccountInfo() async {
    PublicOurchatDatabase db = ourchatAppState.db;
    OurchatDatabase pdb = ourchatAppState.pdb!;
    bool isDataExist = false, isPrivateDataExist = false, needUpdate = false;
    var data = await (db.select(db.publicAccount)
          ..where((u) => u.id.equals(BigInt.from((id.toInt())))))
        .getSingleOrNull();
    if (data != null) {
      isDataExist = true;
      if (isMe) {
        var privateData = await (pdb.select(pdb.account)
              ..where((u) => u.id.equals(BigInt.from((id.toInt())))))
            .getSingleOrNull();
        if (privateData != null) {
          isPrivateDataExist = true;
          GetAccountInfoResponse res = await stub.getAccountInfo(
              GetAccountInfoRequest(
                  id: id,
                  requestValues: [RequestValues.REQUEST_VALUES_UPDATE_TIME]));
          if (res.updateTime.seconds * 1000 ==
              privateData.updateTime.millisecondsSinceEpoch) {
            // 数据库已存在信息且未过期
            ocid = data.ocid;
            avatarKey = data.avatarKey;
            publicUpdateTime =
                OurchatTime(inputDatetime: data.publicUpdateTime);
            username = data.username;
            status = data.status;
            updateTime = OurchatTime(inputDatetime: privateData.updateTime);
            sessions = [];
            var sessionsDynamic = jsonDecode(privateData.sessionsJson);
            for (var i = 0; i < sessionsDynamic.length; i++) {
              sessions.add(Int64.parseInt(sessionsDynamic[i]));
            }
            friends = [];
            var friendsDynamic = jsonDecode(privateData.friendsJson);
            for (var i = 0; i < friendsDynamic.length; i++) {
              friends.add(Int64.parseInt(friendsDynamic[i]));
            }
            email = privateData.email;
            registerTime = OurchatTime(inputDatetime: privateData.registerTime);
            latestMsgTime =
                OurchatTime(inputDatetime: privateData.latestMsgTime);
            gotInfo = true;
            return;
          }
          needUpdate = true;
        }
      }
      if (!needUpdate) {
        GetAccountInfoResponse res = await stub.getAccountInfo(
            GetAccountInfoRequest(id: id, requestValues: [
          RequestValues.REQUEST_VALUES_PUBLIC_UPDATE_TIME
        ]));
        if (res.publicUpdateTime.seconds * 1000 ==
            data.publicUpdateTime.millisecondsSinceEpoch) {
          // 数据库中存在信息且未过期
          ocid = data.ocid;
          avatarKey = data.avatarKey;
          publicUpdateTime = OurchatTime(inputDatetime: data.publicUpdateTime);
          username = data.username;
          status = data.status;
          gotInfo = true;
          return;
        }
      }
    }

    GetAccountInfoResponse res =
        await stub.getAccountInfo(GetAccountInfoRequest(id: id, requestValues: [
      RequestValues.REQUEST_VALUES_AVATAR_KEY,
      RequestValues.REQUEST_VALUES_USER_NAME,
      RequestValues.REQUEST_VALUES_PUBLIC_UPDATE_TIME,
      RequestValues.REQUEST_VALUES_STATUS,
      RequestValues.REQUEST_VALUES_OCID
    ]));
    avatarKey = res.avatarKey;
    username = res.userName;
    publicUpdateTime = OurchatTime(inputTimestamp: res.publicUpdateTime);
    status = res.status;
    ocid = res.ocid;
    if (isMe) {
      res = await stub.getAccountInfo(
        GetAccountInfoRequest(
          id: id,
          requestValues: [
            RequestValues.REQUEST_VALUES_UPDATE_TIME,
            RequestValues.REQUEST_VALUES_SESSIONS,
            RequestValues.REQUEST_VALUES_FRIENDS,
            RequestValues.REQUEST_VALUES_EMAIL,
            RequestValues.REQUEST_VALUES_REGISTER_TIME,
          ],
        ),
      );
      updateTime = OurchatTime(inputTimestamp: res.updateTime);
      email = res.email;
      friends = res.friends;
      sessions = res.sessions;
      registerTime = OurchatTime(inputTimestamp: res.registerTime);
    } else {
      res = await stub.getAccountInfo(
        GetAccountInfoRequest(
          id: id,
          requestValues: [RequestValues.REQUEST_VALUES_DISPLAY_NAME],
        ),
      );
      displayName = res.displayName;
    }

    if (!isDataExist) {
      db.into(db.publicAccount).insert(PublicAccountData(
          id: BigInt.from(id.toInt()),
          username: username,
          status: status,
          avatarKey: avatarKey,
          publicUpdateTime: publicUpdateTime.datetime,
          ocid: ocid));
    } else {
      (db.update(db.publicAccount)
            ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
          .write(PublicAccountCompanion(
              username: Value(username),
              status: Value(status),
              avatarKey: Value(avatarKey),
              publicUpdateTime: Value(publicUpdateTime.datetime),
              ocid: Value(ocid)));
    }
    if (isMe) {
      if (!isPrivateDataExist) {
        pdb.into(pdb.account).insert(AccountData(
            id: BigInt.from(id.toInt()),
            email: email,
            registerTime: registerTime.datetime,
            updateTime: updateTime.datetime,
            friendsJson: jsonEncode(friends),
            sessionsJson: jsonEncode(sessions),
            latestMsgTime: latestMsgTime.datetime));
      } else {
        (pdb.update(pdb.account)
              ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
            .write(AccountCompanion(
                email: Value(email),
                registerTime: Value(registerTime.datetime),
                updateTime: Value(updateTime.datetime),
                friendsJson: Value(jsonEncode(friends)),
                sessionsJson: Value(jsonEncode(sessions)),
                latestMsgTime: Value(latestMsgTime.datetime)));
      }
    }
    gotInfo = true;
  }

  void updateLatestMsgTime() {
    var pdb = ourchatAppState.pdb!;
    (pdb.update(pdb.account)
          ..where((u) => u.id.equals(BigInt.from(id.toInt()))))
        .write(AccountCompanion(latestMsgTime: Value(latestMsgTime.datetime)));
  }
}
