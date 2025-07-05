import 'dart:convert';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/database.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';

class OurchatEvent {
  Int64 eventId;
  int eventType;
  OurchatAccount sender;
  OurchatTime sendTime;
  Object data;
  OurchatEvent(
      this.eventId, this.eventType, this.sender, this.sendTime, this.data);

  void saveToDB(OurchatDatabase privateDB) async {
    // 将消息存入数据库
    await privateDB.into(privateDB.record).insert(RecordData(
        eventId: BigInt.from(eventId.toInt()),
        eventType: eventType,
        sender: BigInt.from(sender.id.toInt()),
        time: sendTime.datetime,
        data: jsonEncode(data),
        read: 0));
  }
}

class AddFriendApproval extends OurchatEvent {
  String leaveMessage;
  AddFriendApproval(Int64 eventId, OurchatAccount sender, OurchatTime sendTime,
      this.leaveMessage)
      : super(eventId, addFriendApprovalEvent, sender, sendTime,
            {"leave_message": leaveMessage});
}

class OurchatEventSystem {
  OurchatAppState ourchatAppState;
  OurchatEventSystem(this.ourchatAppState);

  void listenEvents() async {
    var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
        interceptors: [ourchatAppState.server!.interceptor!]);
    var res = stub.fetchMsgs(FetchMsgsRequest(
        time: ourchatAppState.thisAccount!.latestMsgTime.timestamp));
    res.listen((event) async {
      ourchatAppState.thisAccount!.latestMsgTime =
          OurchatTime(inputTimestamp: event.time);
      ourchatAppState.thisAccount!.updateLatestMsgTime();
      var row = await (ourchatAppState.privateDB!
              .select(ourchatAppState.privateDB!.record)
            ..where((u) => u.eventId.equals(BigInt.from(event.msgId.toInt()))))
          .getSingleOrNull();
      if (row != null) {
        // 重复事件
        return;
      }
      logger.i("receive evnet(type:${event.whichRespondEventType()})");
      // 创建一个发送者oc账号对象
      OurchatAccount sender = OurchatAccount(ourchatAppState);
      sender.token = ourchatAppState.thisAccount!.token;
      sender.recreateStub();
      OurchatEvent? eventObj;
      switch (event.whichRespondEventType()) {
        case FetchMsgsResponse_RespondEventType.addFriendApproval:
          sender.id = event.addFriendApproval.inviterId;
          eventObj = AddFriendApproval(
              event.msgId,
              sender,
              OurchatTime(inputTimestamp: event.time),
              event.addFriendApproval.leaveMessage);
          break;
        default:
          break;
      }
      if (eventObj != null) {
        eventObj.saveToDB(ourchatAppState.privateDB!);
      } else {
        // event 没有被任何case分支处理，属于未知事件类型
        logger.w("Unknown event type(id:${event.msgId})");
      }
    });
  }

  Future selectFriendApproval() async {
    var row = await (ourchatAppState.privateDB!
            .select(ourchatAppState.privateDB!.record)
          ..where((u) => u.eventType.equals(addFriendApprovalEvent)))
        .get();
    List<AddFriendApproval> eventObjList = [];
    for (int i = 0; i < row.length; i++) {
      OurchatAccount sender = OurchatAccount(ourchatAppState);
      sender.token = ourchatAppState.thisAccount!.token;
      sender.recreateStub();
      sender.id = Int64.parseInt(row[i].sender.toString());
      await sender.getAccountInfo();
      eventObjList.add(AddFriendApproval(
          Int64.parseInt(row[i].eventId.toString()),
          sender,
          OurchatTime(inputDatetime: row[i].time),
          jsonDecode(row[i].data)["leave_message"]));
    }
    return eventObjList;
  }
}
