import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/session.dart' as core_session;
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/recall_vote/v1/recall_vote.pb.dart';
import 'package:ourchat/session/session_record.dart';
import 'test_harness.dart';

/// Session notifier stub that returns canned data without touching the
/// network, so the permission-dependent menu entries can be exercised.
class StubSessionNotifier extends core_session.OurChatSession {
  StubSessionNotifier(this.data);

  final core_session.OcSessionData data;

  @override
  core_session.OcSessionData build(String serverId, Int64 sessionId) => data;
}

core_session.OcSessionData buildSessionData({required List<int> permissions}) =>
    core_session.OcSessionData(
      sessionId: Int64(1),
      name: 'test session',
      description: '',
      createdTime: OurChatTime.fromDatetime(DateTime(2026, 1, 1)),
      updatedTime: OurChatTime.fromDatetime(DateTime(2026, 1, 1)),
      members: [Int64(1), Int64(2)],
      roles: {},
      size: 2,
      myPermissions: permissions,
      lastCheckTime: DateTime(2026, 1, 1),
    );

void main() {
  setUpAll(() {
    registerFallbackValue(StartRecallVoteRequest());
  });

  Future<ProviderContainer> pumpMessage(
    WidgetTester tester, {
    required MockOurChatClient client,
    required List<int> permissions,
    required Int64 senderId,
  }) async {
    final container = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
        overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
        overrideAccount(Int64(2), buildTestAccount(Int64(2), 'bob')),
        core_session.ourChatSessionProvider(testServerId, Int64(1))
            .overrideWith(() => StubSessionNotifier(
                  buildSessionData(permissions: permissions),
                )),
      ],
    );
    addTearDown(container.dispose);
    container.read(thisAccountIdProvider.notifier).setAccountId(Int64(1));
    await tester.pumpWidget(
      buildTestApp(
        container: container,
        child: MessageWidget(
          msg: UserMsg(
            senderId: senderId,
            eventId: Int64(10),
            markdownText: 'hello',
            sessionId: Int64(1),
          ),
          opacity: 1.0,
        ),
      ),
    );
    await tester.pump();
    return container;
  }

  /// Long-press an empty spot inside the message container (the selectable
  /// markdown text competes for the gesture).
  Future<void> openMenu(WidgetTester tester) async {
    final rect = tester.getRect(find.byType(MessageWidget));
    await tester.longPressAt(rect.bottomRight - const Offset(10, 10));
    await tester.pump(const Duration(seconds: 1));
    await tester.pump();
  }

  group('message menu "vote to recall" visibility (issue #33)', () {
    testWidgets(
      'offered for someone else\'s message without the RecallMsg permission',
      (tester) async {
        final client = MockOurChatClient();
        when(
          () => client.startRecallVote(any(), options: any(named: 'options')),
        ).thenAnswer((_) => responseFutureOf(StartRecallVoteResponse(voteId: Int64(7))));

        await pumpMessage(
          tester,
          client: client,
          permissions: [1], // SendMsg only
          senderId: Int64(2),
        );

        await openMenu(tester);
        expect(find.text(l10n.voteRecall), findsOneWidget);

        await tester.tap(find.text(l10n.voteRecall));
        await tester.pump(const Duration(seconds: 1));

        final request = verify(
          () => client.startRecallVote(
            captureAny(),
            options: any(named: 'options'),
          ),
        ).captured.single as StartRecallVoteRequest;
        expect(request.msgId, Int64(10));
        expect(request.sessionId, Int64(1));
      },
    );

    testWidgets('hidden for permission holders (they recall directly)', (
      tester,
    ) async {
      await pumpMessage(
        tester,
        client: MockOurChatClient(),
        permissions: [1, recallMsgPermission],
        senderId: Int64(2),
      );

      await openMenu(tester);
      expect(find.text(l10n.voteRecall), findsNothing);
      expect(find.text(l10n.quote), findsOneWidget);
    });

    testWidgets('hidden for the own message', (tester) async {
      await pumpMessage(
        tester,
        client: MockOurChatClient(),
        permissions: [1],
        senderId: Int64(1), // me
      );

      await openMenu(tester);
      expect(find.text(l10n.voteRecall), findsNothing);
    });
  });
}
