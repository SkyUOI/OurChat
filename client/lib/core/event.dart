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
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'event.g.dart';

class OurChatEvent {
  Int64? eventId;
  int? eventType;
  Int64? senderId;
  Int64? sessionId;
  OurChatTime? sendTime;
  Map? data;
  bool read;

  OurChatEvent({
    this.eventId,
    this.eventType,
    this.senderId,
    this.sessionId,
    this.sendTime,
    this.data,
    this.read = false,
  });

  Future saveToDB(OurChatDatabase privateDB) async {
    var result =
        await (privateDB.select(privateDB.record)
              ..where((u) => u.eventId.equals(BigInt.from(eventId!.toInt()))))
            .getSingleOrNull();
    if (result != null) {
      await (privateDB.update(
        privateDB.record,
      )..where((u) => u.eventId.equals(BigInt.from(eventId!.toInt())))).write(
        RecordCompanion(
          eventId: Value(BigInt.from(eventId!.toInt())),
          eventType: Value(eventType!),
          sender: Value(BigInt.from(senderId!.toInt())),
          sessionId: Value(
            sessionId == null ? null : BigInt.from(sessionId!.toInt()),
          ),
          time: Value(sendTime!.datetime),
          data: Value(jsonEncode(data)),
          read: Value((read ? 1 : 0)),
        ),
      );
      return;
    }
    // 不存在 将消息存入数据库
    await privateDB
        .into(privateDB.record)
        .insert(
          RecordData(
            eventId: BigInt.from(eventId!.toInt()),
            eventType: eventType!,
            sender: BigInt.from(senderId!.toInt()),
            sessionId: sessionId == null
                ? null
                : BigInt.from(sessionId!.toInt()),
            time: sendTime!.datetime,
            data: jsonEncode(data),
            read: (read ? 1 : 0),
          ),
        );
  }

  Future loadFromDB(Ref ref, OurChatDatabase privateDB, RecordData row) async {
    eventId = Int64.parseInt(row.eventId.toString());
    eventType = row.eventType;
    senderId = Int64.parseInt(row.sender.toString());
    // Load sender data via provider (side effect)
    final senderNotifier = ref.read(ourChatAccountProvider(senderId!).notifier);
    senderNotifier.recreateStub();
    await senderNotifier.getAccountInfo();

    if (row.sessionId != null) {
      sessionId = Int64.parseInt(row.sessionId.toString());
      final sessionNotifier = ref.read(
        ourChatSessionProvider(sessionId!).notifier,
      );
      try {
        await sessionNotifier.getSessionInfo();
      } catch (e) {
        logger.w("warning when get session info: ${e.toString()}");
      }
    }
    sendTime = OurChatTime.fromDatetime(row.time);
    data = jsonDecode(row.data);
    read = row.read == 1 ? true : false;
  }

  @override
  bool operator ==(Object other) {
    if (other is OurChatEvent) {
      return other.eventId == eventId;
    }
    return false;
  }

  @override
  int get hashCode => eventId!.toInt();
}

class UserMsg extends OurChatEvent {
  String markdownText;
  List<String> involvedFiles;

  UserMsg({
    Int64? eventId,
    Int64? senderId,
    Int64? sessionId,
    OurChatTime? sendTime,
    this.markdownText = "",
    this.involvedFiles = const [],
  }) : super(
         eventId: eventId,
         eventType: msgEvent,
         senderId: senderId,
         sessionId: sessionId,
         sendTime: sendTime,
         data: {"markdown_text": markdownText, "involved_files": involvedFiles},
       );

  @override
  Future loadFromDB(Ref ref, OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(ref, privateDB, row);
    markdownText = data!["markdown_text"];
    involvedFiles = [];
    for (int i = 0; i < data!["involved_files"].length; i++) {
      involvedFiles.add(data!["involved_files"][i]);
    }
  }

  Future<SendMsgResponse?> send(Ref ref, Int64 targetSessionId) async {
    var stub = ref.read(ourChatServerProvider).newStub();
    try {
      var res = await safeRequest(
        stub.sendMsg,
        SendMsgRequest(
          sessionId: targetSessionId,
          markdownText: markdownText,
          involvedFiles: involvedFiles,
          isEncrypted: false,
        ),
        (GrpcError e) {
          showResultMessage(
            e.code,
            e.message,
            notFoundStatus: l10n.notFound(l10n.session),
            permissionDeniedStatus: l10n.permissionDenied(l10n.send),
          );
        },
        rethrowError: true,
      );
      return res;
    } catch (e) {
      return null;
    }
  }
}

class NewFriendInvitationNotification extends OurChatEvent {
  String? leaveMessage;
  int status;
  Int64? inviteeId;
  Int64? resultEventId;

  NewFriendInvitationNotification({
    Int64? eventId,
    Int64? senderId,
    OurChatTime? sendTime,
    this.leaveMessage,
    this.inviteeId,
    this.status = 0,
    this.resultEventId,
  }) : super(
         eventId: eventId,
         eventType: newFriendInvitationNotificationEvent,
         senderId: senderId,
         sendTime: sendTime,
         data: {
           "leave_message": leaveMessage,
           "invitee": inviteeId?.toInt(),
           "status": status,
           "result_event_id": (resultEventId?.toInt()),
         },
       );

  @override
  Future loadFromDB(Ref ref, OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(ref, privateDB, row);
    leaveMessage = data!["leave_message"];
    final parsedInviteeId = Int64.parseInt(data!["invitee"].toString());
    inviteeId = parsedInviteeId;
    final inviteeNotifier = ref.read(
      ourChatAccountProvider(parsedInviteeId).notifier,
    );
    inviteeNotifier.recreateStub();
    await inviteeNotifier.getAccountInfo();
    status = data!["status"];
    resultEventId = data!["result_event_id"] == null
        ? null
        : Int64.parseInt(data!["result_event_id"].toString());
  }
}

class FriendInvitationResultNotification extends OurChatEvent {
  String? leaveMessage;
  Int64? inviteeId;
  bool? accept;
  List<Int64>? requestEventIds;

  FriendInvitationResultNotification({
    Int64? eventId,
    Int64? senderId,
    OurChatTime? sendTime,
    this.leaveMessage,
    this.inviteeId,
    this.accept,
    this.requestEventIds,
  }) : super(
         eventId: eventId,
         eventType: friendInvitationResultNotificationEvent,
         senderId: senderId,
         sendTime: sendTime,
         data: {
           "leave_message": leaveMessage,
           "invitee": inviteeId!.toInt(),
           "accept": accept,
           "request_event_ids": requestEventIds!
               .map((i64) => i64.toInt())
               .toList(),
         },
       );

  @override
  Future loadFromDB(Ref ref, OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(ref, privateDB, row);
    leaveMessage = data!["leave_message"];
    final parsedInviteeId = Int64.parseInt(data!["invitee"].toString());
    inviteeId = parsedInviteeId;
    final inviteeNotifier = ref.read(
      ourChatAccountProvider(parsedInviteeId).notifier,
    );
    inviteeNotifier.recreateStub();
    await inviteeNotifier.getAccountInfo();
    accept = data!["accept"];
    requestEventIds = data!["request_event_ids"]
        .map((n) => Int64.parseInt(n.toString()))
        .toList();
  }
}

@Riverpod(keepAlive: true)
class OurChatEventSystem extends _$OurChatEventSystem {
  final Map _listeners = {};
  ResponseStream<FetchMsgsResponse>? _connection;
  bool _listening = false;

  @override
  bool build() {
    return false;
  }

  void listenEvents() async {
    stopListening();
    final accountId = ref.read(thisAccountIdProvider)!;
    final thisAccount = ref.read(ourChatAccountProvider(accountId).notifier);
    var stub = ref.read(ourChatServerProvider).newStub();

    _connection = stub.fetchMsgs(
      FetchMsgsRequest(time: thisAccount.getLatestMsgTime().timestamp),
    );
    _listening = true;
    logger.i("start to listen event");
    var saveConnectionStream = _connection!.handleError((e) {
      if (!_listening) return;
      logger.w("Disconnected\nTrying to reconnect in 3 seconds ($e)");
      Timer(Duration(seconds: 3), listenEvents);
    });
    await for (var event in saveConnectionStream) {
      {
        thisAccount.setLatestMsgTime(OurChatTime.fromTimestamp(event.time));
        thisAccount.updateLatestMsgTime();
        var row =
            await (privateDB!.select(privateDB!.record)..where(
                  (u) => u.eventId.equals(BigInt.from(event.msgId.toInt())),
                ))
                .getSingleOrNull();
        if (row != null) {
          // 重复事件
          continue;
        }
        FetchMsgsResponse_RespondEventType eventType = event
            .whichRespondEventType();
        logger.i("receive new event(type:$eventType)");
        OurChatEvent? eventObj;
        switch (eventType) {
          case FetchMsgsResponse_RespondEventType // 收到好友申请
              .newFriendInvitationNotification:
            final senderNotifier = ref.read(
              ourChatAccountProvider(
                event.newFriendInvitationNotification.inviterId,
              ).notifier,
            );
            senderNotifier.recreateStub();
            final inviteeNotifier = ref.read(
              ourChatAccountProvider(
                event.newFriendInvitationNotification.inviteeId,
              ).notifier,
            );
            inviteeNotifier.recreateStub();
            eventObj = NewFriendInvitationNotification(
              eventId: event.msgId,
              senderId: event.newFriendInvitationNotification.inviterId,
              sendTime: OurChatTime.fromTimestamp(event.time),
              leaveMessage: event.newFriendInvitationNotification.leaveMessage,
              inviteeId: event.newFriendInvitationNotification.inviteeId,
            );
            break;
          case FetchMsgsResponse_RespondEventType // 收到好友申请结果
              .friendInvitationResultNotification:
            final senderNotifier = ref.read(
              ourChatAccountProvider(
                event.friendInvitationResultNotification.inviterId,
              ).notifier,
            );
            final inviteeNotifier = ref.read(
              ourChatAccountProvider(
                event.friendInvitationResultNotification.inviteeId,
              ).notifier,
            );
            senderNotifier.recreateStub();
            inviteeNotifier.recreateStub();
            List<NewFriendInvitationNotification> eventObjList =
                await selectNewFriendInvitation();
            List<Int64> requestEventIds = [];
            for (int i = 0; i < eventObjList.length; i++) {
              if ((eventObjList[i].senderId! ==
                          event.friendInvitationResultNotification.inviterId &&
                      eventObjList[i].data!["invitee"] == accountId.toInt()) ||
                  eventObjList[i].senderId! == accountId) {
                eventObjList[i].data!["status"] =
                    (event.friendInvitationResultNotification.status ==
                        AcceptFriendInvitationResult
                            .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS
                    ? 1
                    : 2);
                eventObjList[i].read = true;
                eventObjList[i].data!["result_event_id"] = event.msgId.toInt();
                requestEventIds.add(eventObjList[i].eventId!);
                await eventObjList[i].saveToDB(privateDB!);
              }
            }
            eventObj = FriendInvitationResultNotification(
              eventId: event.msgId,
              senderId: event.friendInvitationResultNotification.inviterId,
              sendTime: OurChatTime.fromTimestamp(event.time),
              leaveMessage:
                  event.friendInvitationResultNotification.leaveMessage,
              inviteeId: event.friendInvitationResultNotification.inviteeId,
              accept:
                  (event.friendInvitationResultNotification.status ==
                      AcceptFriendInvitationResult
                          .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS
                  ? true
                  : false),
              requestEventIds: requestEventIds,
            );
            if (event.friendInvitationResultNotification.status ==
                AcceptFriendInvitationResult
                    .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS) {
              thisAccount.getAccountInfo();
            }
            eventObj.read = true;

          case FetchMsgsResponse_RespondEventType.msg:
            final senderNotifier = ref.read(
              ourChatAccountProvider(event.msg.senderId).notifier,
            );
            senderNotifier.recreateStub();
            eventObj = UserMsg(
              eventId: event.msgId,
              senderId: event.msg.senderId,
              sessionId: event.msg.sessionId,
              sendTime: OurChatTime.fromTimestamp(event.time),
              markdownText: event.msg.markdownText,
              involvedFiles: event.msg.involvedFiles,
            );

          default:
            break;
        }
        if (eventObj != null) {
          await eventObj.saveToDB(privateDB!);
          if (_listeners.containsKey(eventType)) {
            // 通知对应listener
            for (int i = 0; i < _listeners[eventType].length; i++) {
              try {
                _listeners[eventType][i](eventObj);
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
    var rows =
        await (privateDB!.select(privateDB!.record)..where(
              (u) => u.eventType.equals(newFriendInvitationNotificationEvent),
            ))
            .get();
    List<NewFriendInvitationNotification> eventObjList = [];
    for (int i = 0; i < rows.length; i++) {
      NewFriendInvitationNotification eventObj =
          NewFriendInvitationNotification();
      await eventObj.loadFromDB(ref, privateDB!, rows[i]);
      eventObjList.add(eventObj);
    }
    return eventObjList;
  }

  Future<List<UserMsg>> getSessionEvent(
    Int64 targetSessionId, {
    int offset = 0,
    int num = 0,
  }) async {
    var pDB = privateDB!;
    var res =
        await (pDB.select(pDB.record)
              ..where(
                (u) => u.sessionId.equals(BigInt.from(targetSessionId.toInt())),
              )
              ..orderBy([
                (u) =>
                    OrderingTerm(expression: u.time, mode: OrderingMode.desc),
              ])
              ..limit((num == 0 ? 50 : num), offset: offset))
            .get();
    List<UserMsg> msgsList = [];
    for (int i = 0; i < res.length; i++) {
      UserMsg msg = UserMsg();
      await msg.loadFromDB(ref, pDB, res[i]);
      msgsList.add(msg);
    }
    return msgsList;
  }

  void addListener(
    FetchMsgsResponse_RespondEventType eventType,
    Function callback,
  ) {
    if (!_listeners.containsKey(eventType)) {
      _listeners[eventType] = [];
    }
    logger.d("add listener of $eventType");
    _listeners[eventType].add(callback);
  }

  void removeListener(
    FetchMsgsResponse_RespondEventType eventType,
    Function callback,
  ) {
    logger.d("remove listener of $eventType");
    if (_listeners.containsKey(eventType)) {
      _listeners[eventType].remove(callback);
      return;
    }
    logger.d("fail to remove");
  }

  void stopListening() {
    _listening = false;
    if (_connection != null) {
      _connection!.cancel();
    }
  }
}
