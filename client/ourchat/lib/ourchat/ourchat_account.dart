import 'package:grpc/grpc.dart';
import 'package:ourchat/google/protobuf/timestamp.pb.dart';
import 'package:ourchat/service/ourchat/get_account_info/v1/get_account_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:ourchat/config.dart';
import 'package:fixnum/fixnum.dart';

class OurchatAccount {
  Int64 id;
  String ocid, token;
  String? email, username, avatarKey, displayName, status, sessions;
  bool isMe;
  Timestamp? publicUpdateTime, updateTime, registerTime;
  ClientChannel? channel;
  List<String>? friends;

  OurchatAccount(
      OurchatConfig config, this.token, this.id, this.ocid, this.isMe) {
    channel = ClientChannel(config.data!["server_address"],
        port: int.parse(config.data!["ws_port"]),
        options:
            const ChannelOptions(credentials: ChannelCredentials.insecure()));
  }

  void getAccountInfo() async {
    var stub = OurChatServiceClient(channel!);
    GetAccountInfoResponse res = await stub.getAccountInfo(
        GetAccountInfoRequest(id: id, requestValues: [
          RequestValues.REQUEST_VALUES_AVATAR_KEY,
          RequestValues.REQUEST_VALUES_USER_NAME,
          RequestValues.REQUEST_VALUES_PUBLIC_UPDATE_TIME,
          // RequestValues.REQUEST_VALUES_STATUS
        ]),
        options: CallOptions(metadata: {"token": token}));
    avatarKey = res.avatarKey;
    username = res.userName;
    publicUpdateTime = res.publicUpdateTime;
    status = res.status;
    if (isMe) {
      res = await stub.getAccountInfo(
          GetAccountInfoRequest(id: id, requestValues: [
            RequestValues.REQUEST_VALUES_UPDATE_TIME,
            // RequestValues.REQUEST_VALUES_SESSIONS,
            RequestValues.REQUEST_VALUES_FRIENDS,
            RequestValues.REQUEST_VALUES_EMAIL,
            RequestValues.REQUEST_VALUES_REGISTER_TIME,
          ]),
          options: CallOptions(metadata: {"token": token}));
      updateTime = res.updateTime;
      email = res.email;
      friends = res.friends;
      sessions = res.sessions;
      registerTime = res.registerTime;
    } else {
      res = await stub.getAccountInfo(
          GetAccountInfoRequest(
              id: id,
              requestValues: [RequestValues.REQUEST_VALUES_DISPLAY_NAME]),
          options: CallOptions(metadata: {"token": token}));
      displayName = res.displayName;
    }
  }
}
