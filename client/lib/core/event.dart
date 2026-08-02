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
           "quote_msg_id": quoteMsgId?.toInt(),
           "quote_sender_id": quoteSenderId?.toInt(),
           "quote_markdown_text": quoteMarkdownText,
           "quote_involved_files": quoteInvolvedFiles,
         },
       );

  @override
  Future loadFromDB(Ref ref, OurChatDatabase privateDB, RecordData row) async {
    await super.loadFromDB(ref, privateDB, row);
    markdownText = data!["markdown_text"];
    involvedFiles = [];
    for (int i = 0; i < data!["involved_files"].length; i++) {
      involvedFiles.add(data!["involved_files"][i]);
    }
    final quotedMsgId = data!["quote_msg_id"];
    quoteMsgId = (quotedMsgId != null && quotedMsgId != 0)
        ? Int64(quotedMsgId)
        : null;
    final quotedSenderId = data!["quote_sender_id"];
    quoteSenderId = (quotedSenderId != null && quotedSenderId != 0)
        ? Int64(quotedSenderId)
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
            String mdText = event.msg.markdownText;
            List<String> files = event.msg.involvedFiles.toList();
            if (event.msg.isEncrypted) {
              final payload = await ref
                  .read(e2eeStoreProvider.notifier)
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

  // ── E2EE room-key protocol handlers ──────────────────────────────────────

  /// We triggered E2eeizeSession (or a room-key rotation): generate a fresh
  /// symmetric room key for the session. Subsequent SendRoomKey notifications
  /// will wrap this key for each member.
  Future<void> _handleUpdateRoomKey(UpdateRoomKeyNotification n) async {
    final store = ref.read(e2eeStoreProvider.notifier);
    final roomKey = generateRoomKey();
    await store.storeKey(n.sessionId, roomKey);
  }

  /// We are the e2eeize initiator and received a member's public key: wrap our
  /// room key for them and deliver it via the SendRoomKey RPC.
  Future<void> _handleSendRoomKeyNotification(SendRoomKeyNotification n) async {
    final sessionId = n.sessionId;
    final store = ref.read(e2eeStoreProvider.notifier);
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
      final stub = ref.read(ourChatServerProvider).newStub();
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
    final store = ref.read(e2eeStoreProvider.notifier);
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
    final store = ref.read(e2eeStoreProvider.notifier);
    final roomKey = await store.unwrapRoomKey(Uint8List.fromList(n.roomKey));
    if (roomKey == null) {
      logger.w('E2EE: could not unwrap join room key for session $sessionId');
      return;
    }
    await store.storeKey(sessionId, roomKey);
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
    bool fetchFromServer = false,
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
    var stub = ref.read(ourChatServerProvider).newStub();
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
            await (privateDB!.select(privateDB!.record)..where(
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
                .read(e2eeStoreProvider.notifier)
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
          await msg.saveToDB(privateDB!);
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
