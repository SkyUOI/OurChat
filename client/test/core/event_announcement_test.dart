import 'dart:async';
import 'dart:convert';

import 'package:drift/native.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/database.dart' as database;
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/announcement/v1/announcement.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

import '../session/test_harness.dart';

/// A `ClientCall` that hands out a pre-built response stream without any
/// transport (mirrors the harness's `_ValueClientCall` for unary calls).
class _StreamClientCall<T> extends ClientCall<dynamic, T> {
  _StreamClientCall(this._stream)
    : super(
        ClientMethod<dynamic, T>(
          '/fake',
          (request) => const <int>[],
          (bytes) => throw UnimplementedError(),
        ),
        const Stream.empty(),
        CallOptions(),
      );

  final Stream<T> _stream;

  @override
  Stream<T> get response => _stream;

  @override
  Future<Map<String, String>> get headers => Future.value(const {});

  @override
  Future<Map<String, String>> get trailers => Future.value(const {});
}

/// A `ResponseStream` handing out a pre-built stream of responses without any
/// transport, so `fetchMsgs` can be stubbed on a mock client.
class FakeResponseStream<T> extends StreamView<T> implements ResponseStream<T> {
  FakeResponseStream(Stream<T> stream) : super(stream);

  final _StreamClientCall<T> _call = _StreamClientCall<T>(const Stream.empty());

  @override
  ResponseFuture<T> get single => ResponseFuture<T>(_call);

  @override
  Future<Map<String, String>> get headers => Future.value(const {});

  @override
  Future<Map<String, String>> get trailers => Future.value(const {});

  @override
  Future<void> cancel() async {}
}

FetchMsgsResponse buildAnnouncementResponse({
  Int64? msgId,
  String title = 'Hello',
  String content = 'World',
  Int64? publisherId,
}) {
  final id = msgId ?? Int64(33);
  final publisher = publisherId ?? Int64(5);
  return FetchMsgsResponse(
    msgId: id,
    time: Timestamp(seconds: Int64(100)),
    announcementResponse: AnnouncementResponse(
      announcement: Announcement(
        title: title,
        content: content,
        publisherId: publisher,
      ),
      createdAt: Timestamp(seconds: Int64(100)),
      id: id,
    ),
  );
}

void main() {
  setUpAll(() {
    registerFallbackValue(FetchMsgsRequest());
  });

  group('AnnouncementResponseEvent data', () {
    test('constructor writes the announcement fields into the data map', () {
      final event = AnnouncementResponseEvent(
        eventId: Int64(21),
        senderId: Int64(7),
        sendTime: OurChatTime.fromDatetime(DateTime(2026, 1, 1, 12)),
        title: 'Maintenance',
        content: 'The server restarts tonight.',
        publisherId: Int64(7),
      );

      expect(event.eventType, announcementResponseEvent);
      expect(event.sessionId, isNull);
      final json = jsonDecode(jsonEncode(event.data)) as Map<String, dynamic>;
      expect(json['id'], 21);
      expect(json['title'], 'Maintenance');
      expect(json['content'], 'The server restarts tonight.');
      expect(json['publisher_id'], 7);
      expect(json['created_at'], DateTime(2026, 1, 1, 12).toIso8601String());
    });

    test('data map round-trips the loadFromDB field extraction', () {
      final event = AnnouncementResponseEvent(
        eventId: Int64(21),
        sendTime: OurChatTime.fromDatetime(DateTime(2026, 2, 2)),
        title: 'title',
        content: 'content',
        publisherId: Int64(9),
      );
      // Reproduce the loadFromDB field extraction (no DB/Ref needed).
      final data = event.data!;
      expect(data['title'], 'title');
      expect(data['content'], 'content');
      expect(
        data['publisher_id'] == null
            ? null
            : Int64.parseInt(data['publisher_id'].toString()),
        Int64(9),
      );
      expect(
        DateTime.parse(data['created_at'] as String),
        DateTime(2026, 2, 2),
      );
    });

    test('publisher-less announcements keep a null publisher_id', () {
      final event = AnnouncementResponseEvent(
        eventId: Int64(1),
        title: 't',
        content: 'c',
      );
      expect(event.data!['publisher_id'], isNull);
    });
  });

  group('announcementEventFromResponse', () {
    test('maps the FetchMsgsResponse proto onto the event', () {
      final event = announcementEventFromResponse(buildAnnouncementResponse());

      expect(event.eventId, Int64(33));
      expect(event.eventType, announcementResponseEvent);
      expect(event.sessionId, isNull);
      expect(event.title, 'Hello');
      expect(event.content, 'World');
      expect(event.publisherId, Int64(5));
      expect(
        event.sendTime!.datetime,
        DateTime.fromMicrosecondsSinceEpoch(100 * 1000000),
      );
      // The data map (persisted to the DB) carries the same values.
      expect(event.data!['id'], 33);
      expect(event.data!['publisher_id'], 5);
    });
  });

  group('event system dispatch', () {
    test(
      'announcement responses are dispatched to registered listeners and saved to the DB',
      () async {
        final client = MockOurChatClient();
        when(
          () => client.fetchMsgs(any(), options: any(named: 'options')),
        ).thenAnswer(
          (_) => FakeResponseStream(Stream.value(buildAnnouncementResponse())),
        );

        final server = FakeOurChatServer(client);
        final db = database.OurChatDatabase(
          testServerId,
          Int64(1),
          NativeDatabase.memory(),
        );
        addTearDown(db.close);

        final container = ProviderContainer(
          overrides: [ourChatServerProvider.overrideWithValue(server)],
        );
        addTearDown(container.dispose);
        container
            .read(instancesProvider.notifier)
            .add(
              OurChatInstance(
                serverId: testServerId,
                accountId: Int64(1),
                server: server,
                privateDB: db,
              ),
            );

        final notifier = container.read(
          ourChatEventSystemProvider(testServerId, Int64(1)).notifier,
        );
        final received = <AnnouncementResponseEvent>[];
        notifier.addListener(
          FetchMsgsResponse_RespondEventType.announcementResponse,
          (AnnouncementResponseEvent event) => received.add(event),
        );

        notifier.listenEvents();
        await pumpEventQueue();

        expect(received.length, 1);
        expect(received.first.title, 'Hello');
        expect(received.first.content, 'World');
        expect(received.first.publisherId, Int64(5));
        expect(received.first.sessionId, isNull);

        // The announcement was persisted as a record row.
        final rows = await db.select(db.record).get();
        expect(rows.length, 1);
        expect(rows.first.eventType, announcementResponseEvent);
        expect(jsonDecode(rows.first.data)['title'], 'Hello');
      },
    );

    test('duplicate announcements are not dispatched twice', () async {
      final client = MockOurChatClient();
      when(
        () => client.fetchMsgs(any(), options: any(named: 'options')),
      ).thenAnswer(
        (_) => FakeResponseStream(
          Stream.fromIterable([
            buildAnnouncementResponse(),
            buildAnnouncementResponse(), // same msgId => duplicate
          ]),
        ),
      );

      final server = FakeOurChatServer(client);
      final db = database.OurChatDatabase(
        testServerId,
        Int64(1),
        NativeDatabase.memory(),
      );
      addTearDown(db.close);

      final container = ProviderContainer(
        overrides: [ourChatServerProvider.overrideWithValue(server)],
      );
      addTearDown(container.dispose);
      container
          .read(instancesProvider.notifier)
          .add(
            OurChatInstance(
              serverId: testServerId,
              accountId: Int64(1),
              server: server,
              privateDB: db,
            ),
          );

      final notifier = container.read(
        ourChatEventSystemProvider(testServerId, Int64(1)).notifier,
      );
      final received = <AnnouncementResponseEvent>[];
      notifier.addListener(
        FetchMsgsResponse_RespondEventType.announcementResponse,
        (AnnouncementResponseEvent event) => received.add(event),
      );

      notifier.listenEvents();
      await pumpEventQueue();

      // The first announcement is dispatched once; the replayed duplicate
      // (same msg id) is skipped by the DB row check.
      expect(received.length, 1);
      final rows = await db.select(db.record).get();
      expect(rows.length, 1);
    });
  });
}
