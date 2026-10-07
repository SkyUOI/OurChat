import 'dart:async';

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/announcement_page.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/announcement/v1/announcement.pb.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

import '../session/test_harness.dart';

/// A `ClientCall`/`ResponseStream` pair that hands out a pre-built stream of
/// responses without any transport (mirrors the announcement event test).
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

FetchMsgsResponse announcementItem(Int64 msgId, String title) =>
    FetchMsgsResponse(
      msgId: msgId,
      time: Timestamp(seconds: Int64(100)),
      announcementResponse: AnnouncementResponse(
        announcement: Announcement(
          title: title,
          content: 'body of $title',
          publisherId: Int64(7),
        ),
        createdAt: Timestamp(seconds: Int64(100)),
        id: msgId,
      ),
    );

void main() {
  setUpAll(() {
    registerFallbackValue(FetchMsgsRequest());
  });

  Future<ProviderContainer> pumpPage(
    WidgetTester tester,
    MockOurChatClient client,
  ) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      buildTestApp(container: container, child: const AnnouncementListPage()),
    );
    await tester.pump();
    return container;
  }

  group('AnnouncementListPage (issue #14)', () {
    testWidgets('lists announcements newest first', (tester) async {
      final client = MockOurChatClient();
      when(
        () => client.fetchMsgs(any(), options: any(named: 'options')),
      ).thenAnswer(
        (_) => FakeResponseStream(
          Stream.fromIterable([
            announcementItem(Int64(1), 'First'),
            announcementItem(Int64(2), 'Second'),
          ]),
        ),
      );

      await pumpPage(tester, client);
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('First'), findsOneWidget);
      expect(find.text('Second'), findsOneWidget);
      expect(find.text('body of First'), findsOneWidget);
      // Newest (last delivered) on top.
      expect(
        tester.getTopLeft(find.text('Second')).dy,
        lessThan(tester.getTopLeft(find.text('First')).dy),
      );
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('shows the empty hint when there are no announcements', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.fetchMsgs(any(), options: any(named: 'options')),
      ).thenAnswer((_) => FakeResponseStream(const Stream.empty()));

      await pumpPage(tester, client);
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text(l10n.noAnnouncement), findsOneWidget);
    });
  });
}
