import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/recall_vote/v1/recall_vote.pb.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'test_harness.dart';

RecallVoteData buildVote({
  Int64? voteId,
  bool settled = false,
  bool passed = false,
  bool? myVote,
  int yesCount = 1,
  int noCount = 0,
  DateTime? deadline,
}) => RecallVoteData(
  voteId: voteId ?? Int64(9),
  sessionId: Int64(1),
  targetMsgId: Int64(77),
  initiatorId: Int64(4),
  yesCount: yesCount,
  noCount: noCount,
  eligibleCount: 2,
  deadline: deadline ?? DateTime.now().add(const Duration(hours: 20)),
  settled: settled,
  passed: passed,
  myVote: myVote,
);

Future<ProviderContainer> pumpTab(
  WidgetTester tester,
  MockOurChatClient client, {
  RecallVoteData? vote,
}) async {
  final container = ProviderContainer(
    overrides: [
      activeAccountTestOverride,
      ourChatServerProvider.overrideWithValue(FakeOurChatServer(client)),
      overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
    ],
  );
  addTearDown(container.dispose);
  container.read(sessionProvider.notifier).state = SessionState(
    tabIndex: TabType.session,
    currentSessionId: Int64(1),
    currentSessionRecords: const [],
    sessionVotes: vote == null ? {} : {vote.voteId: vote},
  );
  await tester.pumpWidget(
    buildTestApp(container: container, child: const SessionTab()),
  );
  await tester.pumpAndSettle(const Duration(seconds: 1));
  return container;
}

void main() {
  setUpAll(() {
    registerFallbackValue(VoteRecallRequest());
  });

  group('vote banner (issue #33)', () {
    testWidgets('no vote for this session renders no banner', (tester) async {
      await pumpTab(tester, MockOurChatClient());
      expect(find.byIcon(Icons.how_to_vote), findsNothing);
    });

    testWidgets('unsettled vote shows the tally and both vote buttons', (
      tester,
    ) async {
      await pumpTab(tester, MockOurChatClient(), vote: buildVote());

      expect(find.byIcon(Icons.how_to_vote), findsOneWidget);
      expect(find.text(l10n.voteRecallBanner), findsOneWidget);
      expect(find.text(l10n.voteYesLabel), findsOneWidget);
      expect(find.text(l10n.voteNoLabel), findsOneWidget);
    });

    testWidgets('tapping "For" calls voteRecall with approve=true', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.voteRecall(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(VoteRecallResponse()));

      await pumpTab(tester, client, vote: buildVote());

      await tester.tap(find.text(l10n.voteYesLabel));
      await tester.pumpAndSettle();

      final request = verify(
        () => client.voteRecall(captureAny(), options: any(named: 'options')),
      ).captured.single as VoteRecallRequest;
      expect(request.voteId, Int64(9));
      expect(request.approve, isTrue);

      // Optimistic local state: the buttons disappear, replaced by the label.
      expect(find.text(l10n.voteVotedYes), findsOneWidget);
    });

    testWidgets('tapping "Against" calls voteRecall with approve=false', (
      tester,
    ) async {
      final client = MockOurChatClient();
      when(
        () => client.voteRecall(any(), options: any(named: 'options')),
      ).thenAnswer((_) => responseFutureOf(VoteRecallResponse()));

      await pumpTab(tester, client, vote: buildVote());

      await tester.tap(find.text(l10n.voteNoLabel));
      await tester.pumpAndSettle();

      final request = verify(
        () => client.voteRecall(captureAny(), options: any(named: 'options')),
      ).captured.single as VoteRecallRequest;
      expect(request.approve, isFalse);
    });

    testWidgets('settled passed vote shows the result and no buttons', (
      tester,
    ) async {
      await pumpTab(
        tester,
        MockOurChatClient(),
        vote: buildVote(settled: true, passed: true),
      );

      expect(find.text(l10n.votePassed), findsOneWidget);
      expect(find.text(l10n.voteYesLabel), findsNothing);
      expect(find.text(l10n.voteNoLabel), findsNothing);
    });

    testWidgets('dismiss removes the banner', (tester) async {
      final container = await pumpTab(
        tester,
        MockOurChatClient(),
        vote: buildVote(),
      );

      await tester.tap(find.byIcon(Icons.close));
      await tester.pump();

      expect(find.byIcon(Icons.how_to_vote), findsNothing);
      expect(container.read(sessionProvider).sessionVotes, isEmpty);
    });

    testWidgets('a settled vote past its deadline stops rendering', (
      tester,
    ) async {
      await pumpTab(
        tester,
        MockOurChatClient(),
        vote: buildVote(
          settled: true,
          passed: false,
          deadline: DateTime.now().subtract(const Duration(hours: 1)),
        ),
      );

      expect(find.byIcon(Icons.how_to_vote), findsNothing);
    });
  });

  group('session vote state helpers', () {
    test('updateSessionVote merges and keeps myVote across tallies', () {
      final container = ProviderContainer(
        overrides: [activeAccountTestOverride],
      );
      addTearDown(container.dispose);
      final notifier = container.read(sessionProvider.notifier);

      notifier.updateSessionVote(buildVote());
      notifier.setMyVote(Int64(9), true);
      expect(
        container.read(sessionProvider).sessionVotes[Int64(9)]!.myVote,
        isTrue,
      );

      // A new tally arrives from the server: myVote survives the merge.
      notifier.updateSessionVote(
        buildVote(yesCount: 2, settled: true, passed: true),
      );
      final merged = container.read(sessionProvider).sessionVotes[Int64(9)]!;
      expect(merged.yesCount, 2);
      expect(merged.settled, isTrue);
      expect(merged.myVote, isTrue);
    });
  });
}
