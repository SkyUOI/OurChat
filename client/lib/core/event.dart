import 'dart:async';
import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/crypto.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/e2ee.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/core/session.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/friends/accept_friend_invitation/v1/accept_friend_invitation.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/allow_user_join_session/v1/allow_user_join_session.pb.dart';
import 'package:ourchat/service/ourchat/session/session_room_key/v1/session_room_key.pb.dart';
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
    // Not present: store the message in the database
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

  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    eventId = Int64.parseInt(row.eventId.toString());
    eventType = row.eventType;
    senderId = Int64.parseInt(row.sender.toString());
    // Load sender data via provider (side effect)
    final senderNotifier = ref.read(
      ourChatAccountProvider(serverId, senderId!).notifier,
    );
    senderNotifier.recreateStub();
    await senderNotifier.getAccountInfo();

    if (row.sessionId != null) {
      sessionId = Int64.parseInt(row.sessionId.toString());
      final sessionNotifier = ref.read(
        ourChatSessionProvider(serverId, sessionId!).notifier,
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
  Int64? quoteMsgId;
  Int64? quoteSenderId;
  String quoteMarkdownText;
  List<String> quoteInvolvedFiles;

  UserMsg({
    Int64? eventId,
    Int64? senderId,
    Int64? sessionId,
    OurChatTime? sendTime,
    this.markdownText = "",
    this.involvedFiles = const [],
    this.quoteMsgId,
    this.quoteSenderId,
    this.quoteMarkdownText = "",
    this.quoteInvolvedFiles = const [],
  }) : super(
         eventId: eventId,
         eventType: msgEvent,
         senderId: senderId,
         sessionId: sessionId,
         sendTime: sendTime,
         data: {
           "markdown_text": markdownText,
           "involved_files": involvedFiles,
           // Int64 values are stored as strings: JSON numbers lose precision
           // above 2^53 on the web (dart2js numbers are doubles), which broke
           // every id comparison against values decoded back from the DB.
           "quote_msg_id": quoteMsgId?.toString(),
           "quote_sender_id": quoteSenderId?.toString(),
           "quote_markdown_text": quoteMarkdownText,
           "quote_involved_files": quoteInvolvedFiles,
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    await super.loadFromDB(ref, serverId, privateDB, row);
    markdownText = data!["markdown_text"];
    involvedFiles = [];
    for (int i = 0; i < data!["involved_files"].length; i++) {
      involvedFiles.add(data!["involved_files"][i]);
    }
    final quotedMsgId = data!["quote_msg_id"];
    quoteMsgId = (quotedMsgId != null && quotedMsgId.toString() != '0')
        ? Int64.parseInt(quotedMsgId.toString())
        : null;
    final quotedSenderId = data!["quote_sender_id"];
    quoteSenderId = (quotedSenderId != null && quotedSenderId.toString() != '0')
        ? Int64.parseInt(quotedSenderId.toString())
        : null;
    quoteMarkdownText = data!["quote_markdown_text"] ?? "";
    quoteInvolvedFiles = [];
    final quotedFiles = data!["quote_involved_files"];
    if (quotedFiles is List) {
      for (int i = 0; i < quotedFiles.length; i++) {
        quoteInvolvedFiles.add(quotedFiles[i].toString());
      }
    }
  }

  /// Map the wire `Msg` proto's quote fields onto nullable `UserMsg` fields.
  /// Zero-valued proto fields (i.e. not set on the wire) map to `null`.
  static ({
    Int64? quoteMsgId,
    Int64? quoteSenderId,
    String quoteMarkdownText,
    List<String> quoteInvolvedFiles,
  })
  quoteFieldsFromMsg(Msg msg) {
    return (
      quoteMsgId: msg.quoteMsgId == Int64.ZERO ? null : msg.quoteMsgId,
      quoteSenderId: msg.quoteSenderId == Int64.ZERO ? null : msg.quoteSenderId,
      quoteMarkdownText: msg.quoteMarkdownText,
      quoteInvolvedFiles: msg.quoteInvolvedFiles.toList(),
    );
  }

  Future<SendMsgResponse?> send(
    OurChatServer server,
    E2eeStore e2eeStore,
    Int64 targetSessionId,
  ) async {
    var stub = server.newStub();
    String wireText = markdownText;
    List<String> wireFiles = involvedFiles;
    bool isEncrypted = false;
    if (e2eeStore.hasKey(targetSessionId)) {
      try {
        wireText = e2eeStore.encryptMessage(
          targetSessionId,
          EncryptedPayload(
            markdownText: markdownText,
            involvedFiles: involvedFiles,
          ),
        );
        wireFiles = const [];
        isEncrypted = true;
      } catch (e) {
        logger.w(
          'E2EE: failed to encrypt outgoing message: $e; sending plaintext',
        );
      }
    }
    try {
      var res = await safeRequest(
        stub.sendMsg,
        SendMsgRequest(
          sessionId: targetSessionId,
          markdownText: wireText,
          involvedFiles: wireFiles,
          isEncrypted: isEncrypted,
          quoteMsgId: quoteMsgId ?? Int64.ZERO,
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
           "invitee": inviteeId?.toString(),
           "status": status,
           "result_event_id": resultEventId?.toString(),
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    await super.loadFromDB(ref, serverId, privateDB, row);
    leaveMessage = data!["leave_message"];
    final parsedInviteeId = Int64.parseInt(data!["invitee"].toString());
    inviteeId = parsedInviteeId;
    final inviteeNotifier = ref.read(
      ourChatAccountProvider(serverId, parsedInviteeId).notifier,
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
           "invitee": inviteeId!.toString(),
           "accept": accept,
           "request_event_ids": requestEventIds!
               .map((i64) => i64.toString())
               .toList(),
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    await super.loadFromDB(ref, serverId, privateDB, row);
    leaveMessage = data!["leave_message"];
    final parsedInviteeId = Int64.parseInt(data!["invitee"].toString());
    inviteeId = parsedInviteeId;
    final inviteeNotifier = ref.read(
      ourChatAccountProvider(serverId, parsedInviteeId).notifier,
    );
    inviteeNotifier.recreateStub();
    await inviteeNotifier.getAccountInfo();
    accept = data!["accept"];
    requestEventIds = data!["request_event_ids"]
        .map((n) => Int64.parseInt(n.toString()))
        .toList();
  }
}

class JoinSessionApprovalNotification extends OurChatEvent {
  Int64? userId;
  String? leaveMessage;
  List<int> publicKey;

  JoinSessionApprovalNotification({
    Int64? eventId,
    Int64? senderId,
    Int64? sessionId,
    OurChatTime? sendTime,
    this.userId,
    this.leaveMessage,
    this.publicKey = const [],
  }) : super(
         eventId: eventId,
         eventType: joinSessionApprovalEvent,
         senderId: senderId,
         sessionId: sessionId,
         sendTime: sendTime,
         data: {
           "user_id": userId?.toString(),
           "leave_message": leaveMessage,
           "public_key": publicKey,
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    await super.loadFromDB(ref, serverId, privateDB, row);
    userId = data!["user_id"] == null
        ? null
        : Int64.parseInt(data!["user_id"].toString());
    leaveMessage = data!["leave_message"];
    publicKey = [];
    final pk = data!["public_key"];
    if (pk is List) {
      for (int i = 0; i < pk.length; i++) {
        publicKey.add(pk[i]);
      }
    }
  }
}

/// A server-wide announcement pushed through the message stream
/// (`FetchMsgsResponse.announcement_response`). Announcements are not bound to
/// any session, so `sessionId` stays null.
class AnnouncementResponseEvent extends OurChatEvent {
  String? title;
  String? content;
  Int64? publisherId;

  AnnouncementResponseEvent({
    Int64? eventId,
    Int64? senderId,
    OurChatTime? sendTime,
    this.title,
    this.content,
    this.publisherId,
  }) : super(
         eventId: eventId,
         eventType: announcementResponseEvent,
         senderId: senderId,
         sendTime: sendTime,
         data: {
           "id": eventId?.toString(),
           "title": title,
           "content": content,
           "publisher_id": publisherId?.toString(),
           "created_at": sendTime?.datetime.toIso8601String(),
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    // Unlike session-bound events, announcements never trigger an account-info
    // fetch for their sender (the publisher may be an unknown admin account).
    eventId = Int64.parseInt(row.eventId.toString());
    eventType = row.eventType;
    senderId = Int64.parseInt(row.sender.toString());
    sendTime = OurChatTime.fromDatetime(row.time);
    data = jsonDecode(row.data);
    read = row.read == 1 ? true : false;
    title = data!["title"];
    content = data!["content"];
    publisherId = data!["publisher_id"] == null
        ? null
        : Int64.parseInt(data!["publisher_id"].toString());
  }
}

/// Map a `FetchMsgsResponse` carrying an `announcement_response` onto an
/// [AnnouncementResponseEvent].
AnnouncementResponseEvent announcementEventFromResponse(
  FetchMsgsResponse event,
) {
  final response = event.announcementResponse;
  return AnnouncementResponseEvent(
    eventId: event.msgId,
    senderId: response.announcement.publisherId,
    sendTime: OurChatTime.fromTimestamp(event.time),
    title: response.announcement.title,
    content: response.announcement.content,
    publisherId: response.announcement.publisherId,
  );
}

/// A recall-vote lifecycle update pushed to a session
/// (`FetchMsgsResponse.recall_vote_notification`, issue #33): a vote was
/// started, a tally changed, or the vote settled (passed → the message gets
/// recalled by the server, failed → nothing happens).
class RecallVoteNotificationEvent extends OurChatEvent {
  Int64 voteId;
  Int64 targetMsgId;
  Int64 initiatorId;
  int yesCount;
  int noCount;
  int eligibleCount;
  DateTime deadline;
  bool settled;
  bool passed;

  RecallVoteNotificationEvent({
    Int64? eventId,
    required this.voteId,
    required Int64 sessionId,
    required this.targetMsgId,
    required this.initiatorId,
    required this.yesCount,
    required this.noCount,
    required this.eligibleCount,
    required this.deadline,
    required this.settled,
    required this.passed,
    OurChatTime? sendTime,
  }) : super(
         eventId: eventId,
         eventType: recallVoteNotificationEvent,
         sessionId: sessionId,
         senderId: initiatorId,
         sendTime: sendTime,
         data: {
           "vote_id": voteId.toString(),
           "msg_id": targetMsgId.toString(),
           "initiator_id": initiatorId.toString(),
           "yes_count": yesCount,
           "no_count": noCount,
           "eligible_count": eligibleCount,
           "deadline": deadline.millisecondsSinceEpoch,
           "settled": settled,
           "passed": passed,
         },
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    eventId = Int64.parseInt(row.eventId.toString());
    eventType = row.eventType;
    senderId = Int64.parseInt(row.sender.toString());
    sessionId = Int64.parseInt(row.sessionId.toString());
    sendTime = OurChatTime.fromDatetime(row.time);
    data = jsonDecode(row.data);
    read = row.read == 1 ? true : false;
    voteId = Int64.parseInt(data!["vote_id"].toString());
    targetMsgId = Int64.parseInt(data!["msg_id"].toString());
    initiatorId = Int64.parseInt(data!["initiator_id"].toString());
    yesCount = data!["yes_count"] as int;
    noCount = data!["no_count"] as int;
    eligibleCount = data!["eligible_count"] as int;
    deadline = DateTime.fromMillisecondsSinceEpoch(data!["deadline"] as int);
    settled = data!["settled"] as bool;
    passed = data!["passed"] as bool;
  }
}

/// Map a `FetchMsgsResponse` carrying a `recall_vote_notification` onto a
/// [RecallVoteNotificationEvent].
RecallVoteNotificationEvent recallVoteEventFromResponse(
  FetchMsgsResponse event,
) {
  final v = event.recallVoteNotification;
  return RecallVoteNotificationEvent(
    eventId: event.msgId,
    voteId: v.voteId,
    sessionId: v.sessionId,
    targetMsgId: v.msgId,
    initiatorId: v.initiatorId,
    yesCount: v.yesCount,
    noCount: v.noCount,
    eligibleCount: v.eligibleCount,
    deadline: v.deadline.toDateTime(),
    settled: v.settled,
    passed: v.passed,
    sendTime: OurChatTime.fromTimestamp(event.time),
  );
}

/// Received by the joiner once a session administrator answers their join
/// request. When accepted, the account data (and therefore the session list)
/// is refreshed so the new conversation shows up immediately.
class AllowUserJoinSessionEvent extends OurChatEvent {
  bool accepted;

  AllowUserJoinSessionEvent({
    Int64? eventId,
    Int64? senderId,
    Int64? sessionId,
    OurChatTime? sendTime,
    this.accepted = false,
  }) : super(
         eventId: eventId,
         eventType: allowUserJoinSessionNotificationEvent,
         senderId: senderId,
         sessionId: sessionId,
         sendTime: sendTime,
         data: {"accepted": accepted},
       );

  @override
  Future loadFromDB(
    Ref ref,
    String serverId,
    OurChatDatabase privateDB,
    RecordData row,
  ) async {
    await super.loadFromDB(ref, serverId, privateDB, row);
    accepted = data!["accepted"];
  }
}

@Riverpod(keepAlive: true)
class OurChatEventSystem extends _$OurChatEventSystem {
  final Map _listeners = {};
  ResponseStream<FetchMsgsResponse>? _connection;
  bool _listening = false;

  @override
  bool build(String serverId, Int64 accountId) {
    return false;
  }

  /// The live instance this event system belongs to.
  OurChatInstance? get _instance =>
      ref.read(instancesProvider)[AccountKey(serverId, accountId)];

  void listenEvents() async {
    stopListening();
    final accountId = this.accountId;
    final inst = _instance;
    if (inst == null) {
      logger.e(
        "event system: no live instance for $serverId/$accountId, not listening",
      );
      return;
    }
    final thisAccount = ref.read(
      ourChatAccountProvider(serverId, accountId).notifier,
    );
    var stub = inst.server.newStub();
    var pDB = inst.privateDB;

    _connection = stub.fetchMsgs(
      FetchMsgsRequest(
        time: thisAccount.getLatestMsgTime().timestamp,
        historyLimit: Int64(200), // Only sync 200 recent messages, then go live
      ),
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
            await (pDB.select(pDB.record)..where(
                  (u) => u.eventId.equals(BigInt.from(event.msgId.toInt())),
                ))
                .getSingleOrNull();
        if (row != null) {
          // Duplicate event
          continue;
        }
        FetchMsgsResponse_RespondEventType eventType = event
            .whichRespondEventType();
        logger.i("receive new event(type:$eventType)");
        OurChatEvent? eventObj;
        switch (eventType) {
          case FetchMsgsResponse_RespondEventType // Received friend request
              .newFriendInvitationNotification:
            final senderNotifier = ref.read(
              ourChatAccountProvider(
                serverId,
                event.newFriendInvitationNotification.inviterId,
              ).notifier,
            );
            senderNotifier.recreateStub();
            final inviteeNotifier = ref.read(
              ourChatAccountProvider(
                serverId,
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
          case FetchMsgsResponse_RespondEventType // Received friend request result
              .friendInvitationResultNotification:
            final senderNotifier = ref.read(
              ourChatAccountProvider(
                serverId,
                event.friendInvitationResultNotification.inviterId,
              ).notifier,
            );
            final inviteeNotifier = ref.read(
              ourChatAccountProvider(
                serverId,
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
                      eventObjList[i].data!["invitee"].toString() ==
                          accountId.toString()) ||
                  eventObjList[i].senderId! == accountId) {
                eventObjList[i].data!["status"] =
                    (event.friendInvitationResultNotification.status ==
                        AcceptFriendInvitationResult
                            .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS
                    ? 1
                    : 2);
                eventObjList[i].read = true;
                eventObjList[i].data!["result_event_id"] = event.msgId
                    .toString();
                requestEventIds.add(eventObjList[i].eventId!);
                await eventObjList[i].saveToDB(pDB);
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
              ourChatAccountProvider(serverId, event.msg.senderId).notifier,
            );
            senderNotifier.recreateStub();
            String mdText = event.msg.markdownText;
            List<String> files = event.msg.involvedFiles.toList();
            if (event.msg.isEncrypted) {
              final payload = await ref
                  .read(e2eeStoreProvider(serverId, accountId).notifier)
                  .decryptMessage(event.msg.sessionId, event.msg.markdownText);
              if (payload != null) {
                mdText = payload.markdownText;
                files = payload.involvedFiles;
              } else {
                // Decryption failed (missing key / tampered). Surface a
                // placeholder so the user knows a message arrived.
                mdText = '[encrypted message]';
                files = const [];
              }
            }
            final quote = UserMsg.quoteFieldsFromMsg(event.msg);
            eventObj = UserMsg(
              eventId: event.msgId,
              senderId: event.msg.senderId,
              sessionId: event.msg.sessionId,
              sendTime: OurChatTime.fromTimestamp(event.time),
              markdownText: mdText,
              involvedFiles: files,
              quoteMsgId: quote.quoteMsgId,
              quoteSenderId: quote.quoteSenderId,
              quoteMarkdownText: quote.quoteMarkdownText,
              quoteInvolvedFiles: quote.quoteInvolvedFiles,
            );

          case FetchMsgsResponse_RespondEventType.receiveRoomKey:
            // A peer (the e2eeize initiator) sent us a wrapped room key.
            await _handleReceiveRoomKey(event.receiveRoomKey);

          case FetchMsgsResponse_RespondEventType.sendRoomKey:
            // We are the e2eeize initiator: the server gave us a member's
            // public key so we can wrap our room key for them.
            await _handleSendRoomKeyNotification(event.sendRoomKey);

          case FetchMsgsResponse_RespondEventType.updateRoomKey:
            // Generate / rotate our room key for this session.
            await _handleUpdateRoomKey(event.updateRoomKey);

          case FetchMsgsResponse_RespondEventType
              .allowUserJoinSessionNotification:
            // We were approved to join a session. If it is E2EE the approver
            // wrapped the room key to our public key — decrypt and store it.
            await _handleAllowUserJoinSession(
              event.allowUserJoinSessionNotification,
            );
            if (event.allowUserJoinSessionNotification.accepted) {
              // Refresh the account data first so the session list picks up
              // the newly joined conversation, then notify listeners.
              await ref
                  .read(ourChatAccountProvider(serverId, accountId).notifier)
                  .getAccountInfo(ignoreCache: true);
              eventObj = AllowUserJoinSessionEvent(
                eventId: event.msgId,
                senderId: accountId,
                sessionId: event.allowUserJoinSessionNotification.sessionId,
                sendTime: OurChatTime.fromTimestamp(event.time),
                accepted: true,
              );
              eventObj.read = true;
            }

          case FetchMsgsResponse_RespondEventType.joinSessionApproval:
            // Somebody asked to join one of our sessions; persist the request
            // and notify listeners (e.g. an approval UI).
            eventObj = JoinSessionApprovalNotification(
              eventId: event.msgId,
              senderId: event.joinSessionApproval.userId,
              sessionId: event.joinSessionApproval.sessionId,
              sendTime: OurChatTime.fromTimestamp(event.time),
              userId: event.joinSessionApproval.userId,
              leaveMessage: event.joinSessionApproval.leaveMessage,
              publicKey: event.joinSessionApproval.publicKey,
            );

          case FetchMsgsResponse_RespondEventType.announcementResponse:
            // A server-wide announcement: persist it and notify listeners
            // (e.g. the foreground announcement dialog).
            eventObj = announcementEventFromResponse(event);

          case FetchMsgsResponse_RespondEventType.recallVoteNotification:
            // Recall-vote lifecycle update (issue #33): persist it and notify
            // the session tab so the vote banner can update.
            eventObj = recallVoteEventFromResponse(event);

          default:
            break;
        }
        if (eventObj != null) {
          await eventObj.saveToDB(pDB);
          if (_listeners.containsKey(eventType)) {
            // Notify the corresponding listeners
            for (int i = 0; i < _listeners[eventType].length; i++) {
              try {
                _listeners[eventType][i](eventObj);
              } catch (e) {
                logger.w("notify listener fail: $e");
              }
            }
          }
        } else {
          // Event not handled by any case branch: unknown event type
          logger.w("Unknown event type(id:${event.msgId})");
        }
      }
    }
  }

  // ── E2EE room-key protocol handlers ──────────────────────────────────────

  /// We triggered E2eeizeSession (or a room-key rotation): generate a fresh
  /// symmetric room key for the session. Subsequent SendRoomKey notifications
  /// will wrap this key for each member.
  Future<void> _handleUpdateRoomKey(UpdateRoomKeyNotification n) async {
    final store = ref.read(e2eeStoreProvider(serverId, accountId).notifier);
    final roomKey = generateRoomKey();
    await store.storeKey(n.sessionId, roomKey);
  }

  /// We are the e2eeize initiator and received a member's public key: wrap our
  /// room key for them and deliver it via the SendRoomKey RPC.
  Future<void> _handleSendRoomKeyNotification(SendRoomKeyNotification n) async {
    final sessionId = n.sessionId;
    final store = ref.read(e2eeStoreProvider(serverId, accountId).notifier);
    // Ensure we have a room key (generate lazily if updateRoomKey was missed).
    Uint8List roomKey;
    final existing = store.keyFor(sessionId) ?? await store.loadKey(sessionId);
    if (existing != null) {
      roomKey = existing;
    } else {
      roomKey = generateRoomKey();
      await store.storeKey(sessionId, roomKey);
    }
    try {
      final wrapped = store.wrapRoomKey(
        roomKey,
        Uint8List.fromList(n.publicKey),
      );
      final stub = _instance!.server.newStub();
      await safeRequest(
        stub.sendRoomKey,
        SendRoomKeyRequest(
          sessionId: n.sessionId,
          userId: n.sender,
          roomKey: wrapped,
        ),
        (GrpcError e) {
          logger.w('SendRoomKey failed: ${e.code} ${e.message}');
        },
      );
    } catch (e) {
      logger.w('E2EE: failed to distribute room key to ${n.sender}: $e');
    }
  }

  /// We received a room key (wrapped to our public key) from a peer: decrypt
  /// it with our private key and store it for the session.
  Future<void> _handleReceiveRoomKey(ReceiveRoomKeyNotification n) async {
    final sessionId = n.sessionId;
    final store = ref.read(e2eeStoreProvider(serverId, accountId).notifier);
    final wrapped = Uint8List.fromList(n.roomKey);
    final roomKey = await store.unwrapRoomKey(wrapped);
    if (roomKey == null) {
      logger.w('E2EE: could not unwrap room key for session $sessionId');
      return;
    }
    await store.storeKey(sessionId, roomKey);
  }

  /// We were approved to join a session. If the approver included a wrapped
  /// room key (E2EE session), decrypt it with our private key and store it so
  /// we can immediately read/write encrypted messages.
  Future<void> _handleAllowUserJoinSession(
    AllowUserJoinSessionNotification n,
  ) async {
    if (!n.accepted) return;
    if (n.roomKey.isEmpty) return;
    final sessionId = n.sessionId;
    final store = ref.read(e2eeStoreProvider(serverId, accountId).notifier);
    final roomKey = await store.unwrapRoomKey(Uint8List.fromList(n.roomKey));
    if (roomKey == null) {
      logger.w('E2EE: could not unwrap join room key for session $sessionId');
      return;
    }
    await store.storeKey(sessionId, roomKey);
  }

  Future selectNewFriendInvitation() async {
    final inst = _instance;
    if (inst == null) return [];
    var pDB = inst.privateDB;
    var rows =
        await (pDB.select(pDB.record)..where(
              (u) => u.eventType.equals(newFriendInvitationNotificationEvent),
            ))
            .get();
    List<NewFriendInvitationNotification> eventObjList = [];
    for (int i = 0; i < rows.length; i++) {
      NewFriendInvitationNotification eventObj =
          NewFriendInvitationNotification();
      await eventObj.loadFromDB(ref, serverId, pDB, rows[i]);
      eventObjList.add(eventObj);
    }
    return eventObjList;
  }

  Future<List<UserMsg>> getSessionEvent(
    Int64 targetSessionId, {
    int offset = 0,
    int num = 0,
    bool fetchFromServer = false,
  }) async {
    final inst = _instance;
    if (inst == null) return [];
    var pDB = inst.privateDB;
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
      await msg.loadFromDB(ref, serverId, pDB, res[i]);
      msgsList.add(msg);
    }

    // If local DB has no results and we're allowed to fetch from server
    if (msgsList.isEmpty && fetchFromServer && offset == 0) {
      final result = await fetchSessionHistoryFromServer(
        targetSessionId,
        OurChatTime.fromDatetime(DateTime.now()),
        limit: num == 0 ? 50 : num,
      );
      return result.messages;
    }

    return msgsList;
  }

  /// Fetch older messages from the server for a specific session.
  /// Returns the list of messages and whether there are more.
  Future<({bool hasMore, List<UserMsg> messages})>
  fetchSessionHistoryFromServer(
    Int64 sessionId,
    OurChatTime beforeTime, {
    int limit = 50,
  }) async {
    final inst = _instance;
    if (inst == null) return (hasMore: false, messages: <UserMsg>[]);
    var stub = inst.server.newStub();
    var pDB = inst.privateDB;
    try {
      var res = await safeRequest(
        stub.fetchSessionHistory,
        FetchSessionHistoryRequest(
          sessionId: sessionId,
          beforeTime: beforeTime.timestamp,
          limit: Int64(limit),
        ),
        (GrpcError e) {
          showResultMessage(
            e.code,
            e.message,
            internalStatus: l10n.serverError,
          );
        },
        rethrowError: true,
      );
      if (res == null) return (hasMore: false, messages: <UserMsg>[]);

      List<UserMsg> msgs = [];
      for (var event in res.messages) {
        if (!event.hasRespondEventType()) continue;

        // Check if already in local DB
        var existing =
            await (pDB.select(pDB.record)..where(
                  (u) => u.eventId.equals(BigInt.from(event.msgId.toInt())),
                ))
                .getSingleOrNull();
        if (existing != null) continue;

        final eventType = event.whichRespondEventType();
        if (eventType == FetchMsgsResponse_RespondEventType.msg) {
          String mdText = event.msg.markdownText;
          List<String> files = event.msg.involvedFiles.toList();
          if (event.msg.isEncrypted) {
            final payload = await ref
                .read(e2eeStoreProvider(serverId, accountId).notifier)
                .decryptMessage(event.msg.sessionId, event.msg.markdownText);
            if (payload != null) {
              mdText = payload.markdownText;
              files = payload.involvedFiles;
            } else {
              mdText = '[encrypted message]';
              files = const [];
            }
          }
          final quote = UserMsg.quoteFieldsFromMsg(event.msg);
          UserMsg msg = UserMsg(
            eventId: Int64(event.msgId),
            senderId: Int64(event.msg.senderId),
            sessionId: Int64(event.msg.sessionId),
            sendTime: OurChatTime.fromTimestamp(event.time),
            markdownText: mdText,
            involvedFiles: files,
            quoteMsgId: quote.quoteMsgId,
            quoteSenderId: quote.quoteSenderId,
            quoteMarkdownText: quote.quoteMarkdownText,
            quoteInvolvedFiles: quote.quoteInvolvedFiles,
          );
          await msg.saveToDB(pDB);
          msgs.add(msg);
        }
      }
      return (hasMore: res.hasMore as bool, messages: msgs);
    } catch (e) {
      logger.w("Failed to fetch session history: $e");
      return (hasMore: false, messages: <UserMsg>[]);
    }
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
