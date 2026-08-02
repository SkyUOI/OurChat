import 'dart:convert';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';

void main() {
  group('UserMsg quote fields', () {
    test('constructor writes quote fields into the data map', () {
      final msg = UserMsg(
        eventId: Int64(1),
        senderId: Int64(2),
        sessionId: Int64(3),
        sendTime: OurChatTime.fromDatetime(DateTime(2026, 1, 1)),
        markdownText: 'hi',
        involvedFiles: const ['f1'],
        quoteMsgId: Int64(5),
        quoteSenderId: Int64(6),
        quoteMarkdownText: 'quoted',
        quoteInvolvedFiles: const ['q1', 'q2'],
      );

      final json = jsonDecode(jsonEncode(msg.data)) as Map<String, dynamic>;
      expect(json['quote_msg_id'], 5);
      expect(json['quote_sender_id'], 6);
      expect(json['quote_markdown_text'], 'quoted');
      expect(json['quote_involved_files'], ['q1', 'q2']);
    });

    test('constructor omits unset quote fields from the data map', () {
      final msg = UserMsg(eventId: Int64(1), markdownText: 'no quote');
      final json = jsonDecode(jsonEncode(msg.data)) as Map<String, dynamic>;
      expect(json['quote_msg_id'], isNull);
      expect(json['quote_sender_id'], isNull);
    });

    test('data map round-trips quote fields', () {
      final msg = UserMsg(
        markdownText: 'x',
        quoteMsgId: Int64(5),
        quoteSenderId: Int64(6),
        quoteMarkdownText: 'quoted',
        quoteInvolvedFiles: const ['q1'],
      );
      // Reproduce the loadFromDB field extraction (no DB/Ref needed).
      final data = msg.data!;
      final quotedMsgId = data['quote_msg_id'];
      final quotedSenderId = data['quote_sender_id'];
      expect(
        (quotedMsgId != null && quotedMsgId != 0) ? Int64(quotedMsgId) : null,
        Int64(5),
      );
      expect(
        (quotedSenderId != null && quotedSenderId != 0)
            ? Int64(quotedSenderId)
            : null,
        Int64(6),
      );
      expect(data['quote_markdown_text'], 'quoted');
      expect(data['quote_involved_files'], ['q1']);
    });
  });

  group('UserMsg.quoteFieldsFromMsg', () {
    test('maps a populated Msg proto onto quote fields', () {
      final msg = Msg(
        senderId: Int64(2),
        sessionId: Int64(3),
        quoteMsgId: Int64(5),
        quoteSenderId: Int64(6),
        quoteMarkdownText: 'quoted text',
        quoteInvolvedFiles: const ['q1', 'q2'],
      );

      final quote = UserMsg.quoteFieldsFromMsg(msg);
      expect(quote.quoteMsgId, Int64(5));
      expect(quote.quoteSenderId, Int64(6));
      expect(quote.quoteMarkdownText, 'quoted text');
      expect(quote.quoteInvolvedFiles, ['q1', 'q2']);
    });

    test('maps zero-valued proto fields to null', () {
      final msg = Msg(quoteMsgId: Int64.ZERO);

      final quote = UserMsg.quoteFieldsFromMsg(msg);
      expect(quote.quoteMsgId, isNull);
      expect(quote.quoteSenderId, isNull);
      expect(quote.quoteMarkdownText, isEmpty);
      expect(quote.quoteInvolvedFiles, isEmpty);
    });

    test('quoteMsgId flows through a full proto UserMsg conversion', () {
      final msg = Msg(
        senderId: Int64(2),
        sessionId: Int64(3),
        markdownText: 'body',
        quoteMsgId: Int64(7),
        quoteSenderId: Int64(9),
      );

      final quote = UserMsg.quoteFieldsFromMsg(msg);
      final userMsg = UserMsg(
        eventId: Int64(10),
        senderId: msg.senderId,
        sessionId: msg.sessionId,
        markdownText: msg.markdownText,
        quoteMsgId: quote.quoteMsgId,
        quoteSenderId: quote.quoteSenderId,
        quoteMarkdownText: quote.quoteMarkdownText,
        quoteInvolvedFiles: quote.quoteInvolvedFiles,
      );
      expect(userMsg.quoteMsgId, Int64(7));
      expect(userMsg.quoteSenderId, Int64(9));
    });
  });
}
