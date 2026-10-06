import 'dart:async';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/allow_user_join_session/v1/allow_user_join_session.pb.dart';
import 'package:ourchat/service/ourchat/session/get_session_info/v1/get_session_info.pb.dart';
import 'package:ourchat/service/ourchat/session/join_session/v1/join_session.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

import 'helpers/oc_test_app.dart';
import 'helpers/oc_test_user.dart';

/// Contract smoke test for issue #289: a non-member asks to join a session
/// found by its id, the session owner approves, and the joiner is added.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('join session request and approval round-trip', (_) async {
    final app = OcTestApp('localhost', 7777, false);
    if (!await app.probe()) {
      markTestSkipped('server at localhost:7777 is not reachable');
      return;
    }

    final owner = await app.registerUser();
    final member = await app.registerUser();
    final joiner = await app.registerUser();
    final sessionId = await app.createSession(owner, [member], name: 'join-it');
    expect(sessionId, isNot(Int64.ZERO));

    // The joiner is not a member yet — ask to join with a leave message.
    final joinFuture = fetchEventUntil(
      owner,
      (resp) =>
          resp.whichRespondEventType() ==
          FetchMsgsResponse_RespondEventType.joinSessionApproval,
    );
    await joiner.stub.joinSession(
      JoinSessionRequest(sessionId: sessionId, leaveMessage: 'let me in'),
    );

    final approval = (await joinFuture).joinSessionApproval;
    expect(approval.sessionId, sessionId);
    expect(approval.userId, joiner.id);
    expect(approval.leaveMessage, 'let me in');
    // The public key is only carried for E2EE sessions; this session is a
    // plain one, so the field must stay empty.
    expect(approval.publicKey, isEmpty);

    // The owner approves the request; the joiner gets the notification.
    final allowedFuture = fetchEventUntil(
      joiner,
      (resp) =>
          resp.whichRespondEventType() ==
          FetchMsgsResponse_RespondEventType.allowUserJoinSessionNotification,
    );
    await owner.stub.allowUserJoinSession(
      AllowUserJoinSessionRequest(
        sessionId: sessionId,
        userId: joiner.id,
        accepted: true,
      ),
    );

    final allowed = (await allowedFuture).allowUserJoinSessionNotification;
    expect(allowed.sessionId, sessionId);
    expect(allowed.accepted, isTrue);

    // The joiner is now part of the session's member list.
    final info = await owner.stub.getSessionInfo(
      GetSessionInfoRequest(
        sessionId: sessionId,
        queryValues: [QueryValues.QUERY_VALUES_MEMBERS],
      ),
    );
    expect(info.members, contains(joiner.id));

    try {
      await owner.deleteSession(sessionId);
    } catch (_) {}
    await app.dispose();
  });
}

/// Wait for the first streamed event matching [matcher].
Future<FetchMsgsResponse> fetchEventUntil(
  OcTestUser user,
  bool Function(FetchMsgsResponse) matcher, {
  Duration timeout = const Duration(seconds: 15),
}) {
  final done = Completer<FetchMsgsResponse>();
  late final StreamSubscription<FetchMsgsResponse> sub;
  sub = user.stub
      .fetchMsgs(FetchMsgsRequest(time: Timestamp(), historyLimit: Int64(200)))
      .listen(
        (resp) {
          if (!done.isCompleted && matcher(resp)) done.complete(resp);
        },
        onError: (Object e) {
          if (!done.isCompleted) done.completeError(e);
        },
      );
  return done.future.timeout(timeout).whenComplete(() => sub.cancel());
}
