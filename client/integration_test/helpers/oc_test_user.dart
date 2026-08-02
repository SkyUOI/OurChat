import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc_connection_interface.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/delete_session/v1/delete_session.pb.dart';
import 'package:ourchat/service/ourchat/session/new_session/v1/session.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

import 'fetch_msg_builder.dart';

/// An authenticated test user with a token-injected stub. Used for **setup**
/// (provisioning accounts/sessions) via raw gRPC — not for exercising client
/// app logic. For the latter use [LiveClientFixture].
class OcTestUser {
  OcTestUser._({
    required this.id,
    required this.ocid,
    required this.token,
    required this.email,
    required this.password,
    required this.keyPair,
    required ClientChannelBase channel,
  }) {
    stub = OurChatServiceClient(
      channel,
      interceptors: [OurChatInterceptor()..setToken(token)],
    );
  }

  final Int64 id;
  final String ocid;
  final String token;
  final String email;
  final String password;
  final ({Uint8List publicKey, Uint8List privateKey}) keyPair;

  late final OurChatServiceClient stub;

  factory OcTestUser.fromRegister(
    ClientChannelBase channel,
    RegisterResponse res, {
    required String email,
    required String password,
    required ({Uint8List publicKey, Uint8List privateKey}) keyPair,
  }) {
    return OcTestUser._(
      id: res.id,
      ocid: res.ocid,
      token: res.token,
      email: email,
      password: password,
      keyPair: keyPair,
      channel: channel,
    );
  }

  // ── messaging ──

  Future<SendMsgResponse> sendMsg(
    Int64 sessionId,
    String markdownText, {
    List<String> involvedFiles = const [],
    bool isEncrypted = false,
  }) => stub.sendMsg(
    SendMsgRequest(
      sessionId: sessionId,
      markdownText: markdownText,
      involvedFiles: involvedFiles,
      isEncrypted: isEncrypted,
    ),
  );

  Future<SendMsgResponse> sendMsgWithQuote(
    Int64 sessionId,
    String markdownText,
    Int64 quoteMsgId, {
    List<String> involvedFiles = const [],
    bool isEncrypted = false,
  }) => stub.sendMsg(
    SendMsgRequest(
      sessionId: sessionId,
      markdownText: markdownText,
      involvedFiles: involvedFiles,
      isEncrypted: isEncrypted,
      quoteMsgId: quoteMsgId,
    ),
  );

  FetchMsgBuilder fetchMsgs({Int64? historyLimit, Timestamp? since}) =>
      FetchMsgBuilder(stub, historyLimit: historyLimit, since: since);

  // ── session ops ──

  Future<void> deleteSession(Int64 sessionId) =>
      stub.deleteSession(DeleteSessionRequest(sessionId: sessionId));

  Future<NewSessionResponse> createSession(
    List<OcTestUser> members, {
    bool e2ee = false,
    String? name,
  }) => stub.newSession(
    NewSessionRequest(
      members: [id, ...members.map((m) => m.id)],
      e2eeOn: e2ee,
      name: name,
    ),
  );
}
