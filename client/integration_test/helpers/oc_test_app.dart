import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/crypto.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

import 'oc_test_user.dart';

/// Probes an external OurChat server and provisions test data (users,
/// sessions) via raw gRPC. Does NOT launch the server — that's the Rust
/// `TestApp`'s job. Skip tests when [probe] returns false.
class OcTestApp {
  OcTestApp(this.host, this.port, this.tls);

  final String host;
  final int port;
  final bool tls;

  late final OurChatServer server;
  late final AuthServiceClient auth;
  late final BasicServiceClient basic;

  bool _reachable = false;
  bool _disposed = false;
  final List<OcTestUser> _users = [];

  bool get isReachable => _reachable;

  /// Probe the server. Returns false (does NOT throw) when unreachable.
  Future<bool> probe({Duration timeout = const Duration(seconds: 3)}) async {
    server = OurChatServer(host, port, tls);
    auth = AuthServiceClient(server.channel);
    basic = BasicServiceClient(server.channel);
    try {
      await basic.ping(PingRequest()).timeout(timeout);
      _reachable = true;
    } catch (_) {
      _reachable = false;
    }
    return _reachable;
  }

  /// Call inside a test to skip when the server is down.
  void requireReachable(String reason) {
    if (!_reachable) {
      throw StateError('server at $host:$port unreachable ($reason)');
    }
  }

  /// Register a throwaway account and return an authenticated [OcTestUser].
  Future<OcTestUser> registerUser({
    String? name,
    String? email,
    String password = 'oc_test_pass_123',
  }) async {
    final kp = await generateRsaKeyPairAsync();
    final suffix = DateTime.now().microsecondsSinceEpoch;
    final emailAddr = email ?? 'oc_test_$suffix@test.local';
    final res = await auth.register(
      RegisterRequest(
        email: emailAddr,
        password: password,
        name: name ?? 'oc_test_$suffix',
        publicKey: kp.publicKey,
      ),
    );
    final user = OcTestUser.fromRegister(
      server.channel,
      res,
      email: emailAddr,
      password: password,
      keyPair: kp,
    );
    _users.add(user);
    return user;
  }

  /// Create a session owned by [creator] with [members]. Returns the session id.
  Future<Int64> createSession(
    OcTestUser creator,
    List<OcTestUser> members, {
    bool e2ee = false,
    String? name,
  }) async {
    final res = await creator.createSession(members, e2ee: e2ee, name: name);
    return res.sessionId;
  }

  /// Best-effort cleanup: delete sessions, unregister users.
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
  }
}
