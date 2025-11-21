import 'dart:async';
import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/session.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/friends/accept_friend_invitation/v1/accept_friend_invitation.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:grpc/grpc.dart';

class OurChatEvent {
  Int64? eventId;
  int? eventType;
  OurChatAccount? sender;
  OurChatSession? session;
  OurChatTime? sendTime;
  Map? data;
  bool read;
  OurChatAppState ourchatAppState;
  OurChatEvent(this.ourchatAppState,
      {this.eventId,
      this.eventType,
      this.sender,
      this.session,
      this.sendTime,
      this.data,
      this.read = false});

  Future saveToDB(OurChatDatabase privateDB) async {
    var result = await (privateDB.select(privateDB.record)
          ..where((u) => u.eventId.equals(BigInt.from(eventId!.toInt()))))
        .getSingleOrNull();
    if (result != null) {
      await (privateDB.update(privateDB.record)
            ..where((u) => u.eventId.equals(BigInt.from(eventId!.toInt()))))
          .write(RecordCompanion(
              eventId: Value(BigInt.from(eventId!.toInt())),
              eventType: Value(eventType!),
              sender: Value(BigInt.from(sender!.id.toInt())),
              sessionId: Value(session == null
                  ? null
                  : BigInt.from(session!.sessionId.toInt())),
              time: Value(sendTime!.datetime),
              data: Value(jsonEncode(data)),
              read: Value((read ? 1 : 0))));
      return;
    }
    // 不存在 将消息存入数据库
    await privateDB.into(privateDB.record).insert(RecordData(
        eventId: BigInt.from(eventId!.toInt()),
        eventType: eventType!,
        sender: BigInt.from(sender!.id.toInt()),
        sessionId:
            session == null ? null : BigInt.from(session!.sessionId.toInt()),
        time: sendTime!.datetime,
        data: jsonEncode(data),
        read: (read ? 1 : 0)));
  }

  Future loadFromDB(OurChatDatabase privateDB, RecordData row) async {
    eventId = Int64.parseInt(row.eventId.toString());
    eventType = row.eventType;
    sender = OurChatAccount(ourchatAppState);
    sender!.id = Int64.parseInt(row.sender.toString());
    sender!.recreateStub();
    await sender!.getAccountInfo();

    if (row.sessionId != null) {
      Int64 sessionId = Int64.parseInt(row.sessionId.toString());
      session = OurChatSession(ourchatAppState, sessionId);
      try {
        await session!.getSessionInfo();
      } catch (e) {
        logger.w("warning when get session info: ${e.toString()}");
      }
    }
    sendTime = OurChatTime(inputDatetime: row.time);
    data = jsonDecode(row.data);
    read = row.read == 1 ? true : false;
  }
}

class OneMessage {
  int? messageType;
  String? text;
  String? imageKey;
  OneMessage({this.messageType, this.text, this.imageKey});

  Map<String, dynamic> serialize() {
    switch (messageType) {
      case textMsg:
        return {"message_type": messageType, "text": text};
      case imageMsg:
        return {"message_type": messageType, "image_key": imageKey};
      default:
        logger.w("serialize fail: unknown message_type($messageType)");
        return {"message_type": messageType, "error": "unknown message_type"};
    }
  }

  void deserialize(Map<String, dynamic> data) {
    messageType = data["message_type"];
    switch (messageType) {
      case textMsg:
        text = data["text"];
      case imageMsg:
        imageKey = data["image_key"];
      default:
        logger.w("deserialize fail: unknown message_type($messageType)");
    }
  }
}

class BundleMsgs extends OurChatEvent {
  List<OneMessage> msgs;
  BundleMsgs(OurChatAppState ourchatAppState,
      {Int64? eventId,
      OurChatAccount? sender,
      OurChatSession? session,
      OurChatTime? sendTime,
      this.msgs = const []})
      : super(ourchatAppState,
            eventId: eventId,
            eventType: msgEvent,
            sender: sender,
            session: session,
            sendTime: sendTime,
            data: {"msgs": msgs.map((u) => u.serialize()).toList()});

  @override
  Future loadFromDB(OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(privateDB, row);
    msgs = [];
    for (int i = 0; i < data!["msgs"].length; i++) {
      OneMessage oneMessage = OneMessage();
      oneMessage.deserialize(data!["msgs"][i]);
      msgs.add(oneMessage);
    }
  }

  Future<SendMsgResponse?> send(OurChatSession session) async {
    var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
        interceptors: [ourchatAppState.server!.interceptor!]);
    var l10n = ourchatAppState.l10n;
    try {
      var res = await safeRequest(
          stub.sendMsg,
          SendMsgRequest(
              sessionId: session.sessionId,
              bundleMsgs: msgs.map((u) => OneMsg(text: u.text)),
              isEncrypted: false), (GrpcError e) {
        showResultMessage(ourchatAppState, e.code, e.message,
            notFoundStatus: l10n.notFound(l10n.session),
            permissionDeniedStatus: l10n.permissionDenied(l10n.send));
      }, rethrowError: true);
      return res;
    } catch (e) {
      return null;
    }
  }

  OneMessage operator [](int index) {
    return msgs[index];
  }
}

class NewFriendInvitationNotification extends OurChatEvent {
  String? leaveMessage;
  int status;
  OurChatAccount? invitee;
  Int64? resultEventId;
  NewFriendInvitationNotification(OurChatAppState ourchatAppState,
      {Int64? eventId,
      OurChatAccount? sender,
      OurChatTime? sendTime,
      this.leaveMessage,
      this.invitee,
      this.status = 0,
      this.resultEventId})
      : super(ourchatAppState,
            eventId: eventId,
            eventType: newFriendInvitationNotificationEvent,
            sender: sender,
            sendTime: sendTime,
            data: {
              "leave_message": leaveMessage,
              "invitee": invitee?.id.toInt(),
              "status": status,
              "result_event_id": (resultEventId?.toInt())
            });

  @override
  Future loadFromDB(OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(privateDB, row);
    leaveMessage = data!["leave_message"];
    invitee = OurChatAccount(ourchatAppState);
    invitee!.id = Int64.parseInt(data!["invitee"].toString());
    invitee!.recreateStub();
    await invitee!.getAccountInfo();
    status = data!["status"];
    resultEventId = data!["result_event_id"] == null
        ? null
        : Int64.parseInt(data!["result_event_id"].toString());
  }
}

class FriendInvitationResultNotification extends OurChatEvent {
  String? leaveMessage;
  OurChatAccount? invitee;
  bool? accept;
  List<Int64>? requestEventIds;
  FriendInvitationResultNotification(OurChatAppState ourchatAppState,
      {Int64? eventId,
      OurChatAccount? sender,
      OurChatTime? sendTime,
      this.leaveMessage,
      this.invitee,
      this.accept,
      this.requestEventIds})
      : super(ourchatAppState,
            eventId: eventId,
            eventType: friendInvitationResultNotificationEvent,
            sender: sender,
            sendTime: sendTime,
            data: {
              "leave_message": leaveMessage,
              "invitee": invitee!.id.toInt(),
              "accept": accept,
              "request_event_ids":
                  requestEventIds!.map((i64) => i64.toInt()).toList()
            });

  @override
  Future loadFromDB(OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(privateDB, row);
    leaveMessage = data!["leave_message"];
    invitee = OurChatAccount(ourchatAppState);
    invitee!.id = Int64.parseInt(data!["invitee"].toString());
    invitee!.recreateStub();
    await invitee!.getAccountInfo();
    accept = data!["accept"];
    requestEventIds = data!["request_event_ids"]
        .map((n) => Int64.parseInt(n.toString()))
        .toList();
  }
}

class OurChatEventSystem {
  OurChatAppState ourchatAppState;
  Map listeners = {};
  OurChatEventSystem(this.ourchatAppState);
  ResponseStream<FetchMsgsResponse>? connection;
  bool listening = false;

  void listenEvents() async {
    stopListening();
    var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
        interceptors: [ourchatAppState.server!.interceptor!]);

    connection = stub.fetchMsgs(FetchMsgsRequest(
        time: ourchatAppState.thisAccount!.latestMsgTime.timestamp));
    listening = true;
    logger.i("start to listen event");
    var saveConnectionStream = connection!.handleError((e) {
      if (!listening) return;
      logger.w("Disconnected\nTrying to reconnect in 3 seconds ($e)");
      Timer(Duration(seconds: 3), listenEvents);
    });
    await for (var event in saveConnectionStream) {
      {
        ourchatAppState.thisAccount!.latestMsgTime =
            OurChatTime(inputTimestamp: event.time);
        ourchatAppState.thisAccount!.updateLatestMsgTime();
        var row = await (ourchatAppState.privateDB!
                .select(ourchatAppState.privateDB!.record)
              ..where(
                  (u) => u.eventId.equals(BigInt.from(event.msgId.toInt()))))
            .getSingleOrNull();
        if (row != null) {
          // 重复事件
          continue;
        }
        FetchMsgsResponse_RespondEventType eventType =
            event.whichRespondEventType();
        logger.i("receive new event(type:$eventType)");
        // 创建一个发送者oc账号对象
        OurChatAccount sender = OurChatAccount(ourchatAppState);
        sender.recreateStub();
        OurChatEvent? eventObj;
        switch (eventType) {
          case FetchMsgsResponse_RespondEventType // 收到好友申请
                .newFriendInvitationNotification:
            sender.id = event.newFriendInvitationNotification.inviterId;
            OurChatAccount invitee = OurChatAccount(ourchatAppState);
            invitee.id = event.newFriendInvitationNotification.inviteeId;
            eventObj = NewFriendInvitationNotification(ourchatAppState,
                eventId: event.msgId,
                sender: sender,
                sendTime: OurChatTime(inputTimestamp: event.time),
                leaveMessage:
                    event.newFriendInvitationNotification.leaveMessage,
                invitee: invitee);
            break;
          case FetchMsgsResponse_RespondEventType // 收到好友申请结果
                .friendInvitationResultNotification:
            OurChatAccount invitee = OurChatAccount(ourchatAppState);
            sender.id = event.friendInvitationResultNotification.inviterId;
            invitee.id = event.friendInvitationResultNotification.inviteeId;
            invitee.recreateStub();
            List<NewFriendInvitationNotification> eventObjList =
                await selectNewFriendInvitation();
            List<Int64> requestEventIds = [];
            for (int i = 0; i < eventObjList.length; i++) {
              if ((eventObjList[i].sender!.id == sender.id &&
                      eventObjList[i].data!["invitee"] ==
                          ourchatAppState.thisAccount!.id.toInt()) ||
                  eventObjList[i].sender!.id ==
                      ourchatAppState.thisAccount!.id) {
                eventObjList[i].data!["status"] =
                    (event.friendInvitationResultNotification.status ==
                            AcceptFriendInvitationResult
                                .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS
                        ? 1
                        : 2);
                eventObjList[i].read = true;
                eventObjList[i].data!["result_event_id"] = event.msgId.toInt();
                requestEventIds.add(eventObjList[i].eventId!);
                await eventObjList[i].saveToDB(ourchatAppState.privateDB!);
              }
            }
            eventObj = FriendInvitationResultNotification(ourchatAppState,
                eventId: event.msgId,
                sender: sender,
                sendTime: OurChatTime(inputTimestamp: event.time),
                leaveMessage:
                    event.friendInvitationResultNotification.leaveMessage,
                invitee: invitee,
                accept: (event.friendInvitationResultNotification.status ==
                        AcceptFriendInvitationResult
                            .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS
                    ? true
                    : false),
                requestEventIds: requestEventIds);
            if (event.friendInvitationResultNotification.status ==
                AcceptFriendInvitationResult
                    .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS) {
              ourchatAppState.thisAccount!.getAccountInfo();
            }
            eventObj.read = true;

          case FetchMsgsResponse_RespondEventType.msg:
            sender.id = event.msg.senderId;
            List<OneMessage> msgs = [];
            for (int i = 0; i < event.msg.bundleMsgs.length; i++) {
              OneMsg oneMsg = event.msg.bundleMsgs[i]; // OneMsg 为grpc对象
              OneMessage oneMessage = OneMessage(); // OneMessage为OurChat对象
              if (oneMsg.text.isNotEmpty) {
                oneMessage.messageType = textMsg;
                oneMessage.text = oneMsg.text;
              } else if (oneMsg.image.isNotEmpty) {
                oneMessage.messageType = imageMsg;
                oneMessage.imageKey = oneMsg.image;
              }
              msgs.add(oneMessage);
            }
            eventObj = BundleMsgs(ourchatAppState,
                eventId: event.msgId,
                sender: sender,
                session: OurChatSession(ourchatAppState, event.msg.sessionId),
                sendTime: OurChatTime(inputTimestamp: event.time),
                msgs: msgs);

          default:
            break;
        }
        if (eventObj != null) {
          await eventObj.saveToDB(ourchatAppState.privateDB!);
          if (listeners.containsKey(eventType)) {
            // 通知对应listener
            for (int i = 0; i < listeners[eventType].length; i++) {
              try {
                listeners[eventType][i](eventObj);
              } catch (e) {
                logger.w("notify listener fail: $e");
              }
            }
          }
        } else {
          // event 没有被任何case分支处理，属于未知事件类型
          logger.w("Unknown event type(id:${event.msgId})");
        }
      }
    }
  }

  Future selectNewFriendInvitation() async {
    var rows = await (ourchatAppState.privateDB!
            .select(ourchatAppState.privateDB!.record)
          ..where(
              (u) => u.eventType.equals(newFriendInvitationNotificationEvent)))
        .get();
    List<NewFriendInvitationNotification> eventObjList = [];
    for (int i = 0; i < rows.length; i++) {
      NewFriendInvitationNotification eventObj =
          NewFriendInvitationNotification(ourchatAppState);
      await eventObj.loadFromDB(ourchatAppState.privateDB!, rows[i]);
      eventObjList.add(eventObj);
    }
    return eventObjList;
  }

  Future<List<BundleMsgs>> getSessionEvent(
      OurChatAppState ourchatAppState, OurChatSession session,
      {int offset = 0, int num = 0}) async {
    var privateDB = ourchatAppState.privateDB!;
    var res = await (privateDB.select(privateDB.record)
          ..where(
              (u) => u.sessionId.equals(BigInt.from(session.sessionId.toInt())))
          ..orderBy([
            (u) => OrderingTerm(expression: u.time, mode: OrderingMode.desc)
          ])
          ..limit((num == 0 ? 50 : num), offset: offset))
        .get();
    List<BundleMsgs> bundleMsgsList = [];
    for (int i = 0; i < res.length; i++) {
      BundleMsgs bundleMsgs = BundleMsgs(ourchatAppState);
      await bundleMsgs.loadFromDB(privateDB, res[i]);
      bundleMsgsList.add(bundleMsgs);
    }
    return bundleMsgsList;
  }

  void addListener(
      FetchMsgsResponse_RespondEventType eventType, Function callback) {
    if (!listeners.containsKey(eventType)) {
      listeners[eventType] = [];
    }
    logger.d("add listener of $eventType");
    listeners[eventType].add(callback);
  }

  void removeListener(
      FetchMsgsResponse_RespondEventType eventType, Function callback) {
    logger.d("remove listener of $eventType");
    if (listeners.containsKey(eventType)) {
      listeners[eventType].remove(callback);
      return;
    }
    logger.d("fail to remove");
  }

  void stopListening() {
    listening = false;
    if (connection != null) {
      connection!.cancel();
    }
  }
}
