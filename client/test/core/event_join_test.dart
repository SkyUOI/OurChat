import 'dart:convert';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';

void main() {
  group('JoinSessionApprovalNotification', () {
    test('constructor writes the approval fields into the data map', () {
      final event = JoinSessionApprovalNotification(
        eventId: Int64(11),
        senderId: Int64(9),
        sessionId: Int64(3),
        sendTime: OurChatTime.fromDatetime(DateTime(2026, 1, 1)),
        userId: Int64(9),
        leaveMessage: 'please let me in',
        publicKey: const [1, 2, 3],
      );

      expect(event.eventType, joinSessionApprovalEvent);
      final json = jsonDecode(jsonEncode(event.data)) as Map<String, dynamic>;
      expect(json['user_id'], '9');
      expect(json['leave_message'], 'please let me in');
      expect(json['public_key'], [1, 2, 3]);
    });

    test('data map round-trips the loadFromDB field extraction', () {
      final event = JoinSessionApprovalNotification(
        eventId: Int64(11),
        senderId: Int64(9),
        sessionId: Int64(3),
        userId: Int64(9),
        leaveMessage: 'hello',
        publicKey: const [4, 5],
      );
      // Reproduce the loadFromDB field extraction (no DB/Ref needed).
      final data = event.data!;
      final userId = data['user_id'] == null
          ? null
          : Int64.parseInt(data['user_id'].toString());
      expect(userId, Int64(9));
      expect(data['leave_message'], 'hello');
      final pk = data['public_key'];
      expect(pk is List && pk.length == 2 && pk[0] == 4 && pk[1] == 5, isTrue);
    });
  });

  group('AllowUserJoinSessionEvent', () {
    test('records the acceptance and the event type', () {
      final event = AllowUserJoinSessionEvent(
        eventId: Int64(12),
        senderId: Int64(1),
        sessionId: Int64(3),
        accepted: true,
      );

      expect(event.eventType, allowUserJoinSessionNotificationEvent);
      expect(event.data!['accepted'], isTrue);
      expect(event.sessionId, Int64(3));
    });
  });
}
