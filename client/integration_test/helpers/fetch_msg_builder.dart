import 'dart:async';

import 'package:fixnum/fixnum.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';

/// Streams messages from the server with timeout, mirroring Rust's
/// `FetchMsgBuilder`.
///
/// ```
/// final msgs = await user.fetchMsgs().fetch(2);
/// final m = await user.fetchMsgs().fetchUntil((m) => m.markdownText == 'x');
/// ```
class FetchMsgBuilder {
  FetchMsgBuilder(this._stub, {Int64? historyLimit, Timestamp? since})
    : _historyLimit = historyLimit ?? Int64(200),
      _since = since ?? Timestamp();

  final OurChatServiceClient _stub;
  final Int64 _historyLimit;
  final Timestamp? _since;
  Duration _timeout = const Duration(seconds: 15);

  FetchMsgBuilder setTimeout(Duration d) {
    _timeout = d;
    return this;
  }

  /// Collect exactly [count] msg-type events, or throw [TimeoutException].
  Future<List<Msg>> fetch(int count) async {
    final collected = <Msg>[];
    await _run((resp) {
      if (resp.whichRespondEventType() ==
          FetchMsgsResponse_RespondEventType.msg) {
        collected.add(resp.msg);
        return collected.length >= count;
      }
      return false;
    });
    return collected;
  }

  /// Collect until [matcher] returns true (returns all collected so far).
  Future<List<Msg>> fetchUntil(bool Function(Msg) matcher) async {
    final collected = <Msg>[];
    await _run((resp) {
      if (resp.whichRespondEventType() ==
          FetchMsgsResponse_RespondEventType.msg) {
        collected.add(resp.msg);
        return matcher(resp.msg);
      }
      return false;
    });
    return collected;
  }

  Future<void> _run(bool Function(FetchMsgsResponse) stop) async {
    final done = Completer<void>();
    final sub = _stub
        .fetchMsgs(FetchMsgsRequest(time: _since, historyLimit: _historyLimit))
        .listen(
          (resp) {
            if (!done.isCompleted && stop(resp)) done.complete();
          },
          onError: (Object e) {
            if (!done.isCompleted) done.completeError(e);
          },
        );
    try {
      await done.future.timeout(_timeout);
    } finally {
      await sub.cancel();
    }
  }
}
