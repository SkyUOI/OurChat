import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

/// Regression test for the gRPC-Web CORS bug (found by manually testing the
/// Flutter web build): gRPC-Web "trailers-only" error responses carry
/// grpc-status/grpc-message in the HTTP response headers, and browsers hide
/// those from cross-origin JavaScript unless the server lists them in
/// `Access-Control-Expose-Headers`. Before the server exposed them, every
/// remote error surfaced to the web client as an opaque UNKNOWN(2) — e.g.
/// searching for a nonexistent user reported "unknown error" instead of
/// "user not found".
///
/// Runs against a real server (default localhost:7777, override with
/// `--dart-define=OURCHAT_TEST_SERVER_HOST/PORT`); skips when unreachable.
/// On Chrome via `flutter drive` the app is served on a different port, so
/// the request is genuinely cross-origin and exercises the CORS exposure;
/// on other platforms it validates that the real status survives the
/// transport at all.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final host = const String.fromEnvironment(
    'OURCHAT_TEST_SERVER_HOST',
    defaultValue: 'localhost',
  );
  final port =
      int.tryParse(const String.fromEnvironment('OURCHAT_TEST_SERVER_PORT')) ??
      7777;

  testWidgets('remote error keeps its real gRPC status over gRPC-Web', (
    _,
  ) async {
    final server = OurChatServer(host, port, false);
    final reachable = await server.getServerInfo() == okStatusCode;
    if (!reachable) {
      markTestSkipped('no server reachable at $host:$port');
      return;
    }

    final stub = BasicServiceClient(server.channel, interceptors: []);
    try {
      await stub.getId(GetIdRequest(ocid: 'no-such-ocid-web-test'));
      fail('getId for a nonexistent ocid must fail');
    } on grpc.GrpcError catch (e) {
      expect(
        e.codeName,
        isNot('UNKNOWN'),
        reason: kIsWeb
            ? 'gRPC-Web trailers-only errors must surface their real status; '
                  'UNKNOWN here means the server is not exposing grpc-status '
                  'via Access-Control-Expose-Headers'
            : 'the real error status must survive the transport',
      );
      expect(e.code, notFoundStatusCode);
    }
  });
}
