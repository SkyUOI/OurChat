import 'dart:convert';

import 'package:ourchat/const.dart';
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
  late Int64 id;
  late String username, avatarKey, displayName, status, email, ocid, token;
  bool isMe = false, gotInfo = false;
  late Timestamp publicUpdateTime, updateTime, registerTime;
  OurChatServer server;
  late List<Int64> friends, sessions;
  late OurChatServiceClient stub;

  OurchatAccount(this.server) {
    stub = OurChatServiceClient(server.channel!);
  }

  void recreateStub() {
    var interceptor = AuthInterceptor();
    interceptor.setToken(token);
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
    publicUpdateTime = res.publicUpdateTime;
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
      updateTime = res.updateTime;
      email = res.email;
      friends = res.friends;
      sessions = res.sessions;
      registerTime = res.registerTime;
    } else {
      res = await stub.getAccountInfo(
        GetAccountInfoRequest(
          id: id,
          requestValues: [RequestValues.REQUEST_VALUES_DISPLAY_NAME],
        ),
      );
      displayName = res.displayName;
    }
    gotInfo = true;
  }
}
