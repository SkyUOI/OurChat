import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/service/ourchat/msg_delivery/recall_vote/v1/recall_vote.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

/// Tests for the recall-vote notification event (issue #33): the data map it
/// persists and its mapping from the gRPC response.
void main() {
  group('RecallVoteNotificationEvent data', () {
    test('constructor writes the vote fields into the data map', () {
      final deadline = DateTime(2026, 10, 7, 12);
      final event = RecallVoteNotificationEvent(
        eventId: Int64(50),
        voteId: Int64(9),
        sessionId: Int64(3),
        targetMsgId: Int64(77),
        initiatorId: Int64(4),
        yesCount: 1,
        noCount: 0,
        eligibleCount: 2,
        deadline: deadline,
        settled: false,
        passed: false,
      );

      expect(event.eventType, recallVoteNotificationEvent);
      expect(event.sessionId, Int64(3));
      final data = event.data!;
      expect(data['vote_id'], '9');
      expect(data['msg_id'], '77');
      expect(data['initiator_id'], '4');
      expect(data['yes_count'], 1);
      expect(data['no_count'], 0);
      expect(data['eligible_count'], 2);
      expect(data['deadline'], deadline.millisecondsSinceEpoch);
      expect(data['settled'], false);
      expect(data['passed'], false);
    });

    test('data map values round-trip the loadFromDB extraction', () {
      final deadline = DateTime(2026, 10, 7, 12);
      final event = RecallVoteNotificationEvent(
        eventId: Int64(51),
        voteId: Int64(10),
        sessionId: Int64(3),
        targetMsgId: Int64(78),
        initiatorId: Int64(5),
        yesCount: 2,
        noCount: 1,
        eligibleCount: 2,
        deadline: deadline,
        settled: true,
        passed: true,
      );
      final data = event.data!;
      expect(Int64.parseInt(data['vote_id'].toString()), Int64(10));
      expect(data['yes_count'] as int, 2);
      expect(
        DateTime.fromMillisecondsSinceEpoch(data['deadline'] as int),
        deadline,
      );
      expect(data['settled'] as bool, true);
      expect(data['passed'] as bool, true);
    });
  });

  group('recallVoteEventFromResponse', () {
    test('maps the FetchMsgsResponse proto onto the event', () {
      final deadline = Timestamp.fromDateTime(DateTime.utc(2026, 10, 8));
      final response = FetchMsgsResponse(
        msgId: Int64(60),
        time: Timestamp(seconds: Int64(100)),
        recallVoteNotification: RecallVoteNotification(
          voteId: Int64(11),
          msgId: Int64(79),
          sessionId: Int64(3),
          initiatorId: Int64(6),
          yesCount: 1,
          noCount: 1,
          eligibleCount: 3,
          deadline: deadline,
          settled: false,
          passed: false,
        ),
      );

      final event = recallVoteEventFromResponse(response);

      expect(event.eventId, Int64(60));
      expect(event.eventType, recallVoteNotificationEvent);
      expect(event.voteId, Int64(11));
      expect(event.targetMsgId, Int64(79));
      expect(event.sessionId, Int64(3));
      expect(event.initiatorId, Int64(6));
      expect(event.yesCount, 1);
      expect(event.noCount, 1);
      expect(event.eligibleCount, 3);
      expect(event.deadline, DateTime.utc(2026, 10, 8));
      expect(event.settled, false);
      expect(event.passed, false);
      expect(
        event.sendTime!.datetime,
        DateTime.fromMicrosecondsSinceEpoch(100 * 1000000),
      );
    });
  });
}
