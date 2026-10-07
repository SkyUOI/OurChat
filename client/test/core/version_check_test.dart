import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/version_check.dart';
import 'package:ourchat/service/basic/server/v1/server.pb.dart';

ServerVersion _v(int major, int minor, int patch) =>
    ServerVersion(major: major, minor: minor, patch: patch);

void main() {
  group('parseVersionString', () {
    test('parses plain and v-prefixed versions', () {
      expect(parseVersionString('v1.2.3'), (major: 1, minor: 2, patch: 3));
      expect(parseVersionString('1.2.3'), (major: 1, minor: 2, patch: 3));
    });

    test('ignores pre-release suffixes', () {
      expect(parseVersionString('v0.0.0.beta'), (major: 0, minor: 0, patch: 0));
      expect(parseVersionString('v2.5.0-rc.1'), (major: 2, minor: 5, patch: 0));
    });

    test('fills missing parts with zeros and rejects no-number input', () {
      expect(parseVersionString('v3'), (major: 3, minor: 0, patch: 0));
      expect(parseVersionString('beta'), isNull);
    });
  });

  group('checkServerCompatibility (issue #16)', () {
    test('ok when server meets the client minimum and no client floor set', () {
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 1, 0),
          minimumClientVersion: null,
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.ok,
      );
    });

    test('server too old when below the client-supported floor', () {
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 0, 9),
          minimumClientVersion: null,
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.serverTooOld,
      );
    });

    test('server 0.0.0 floor means no client restriction', () {
      expect(minimumClientVersionApplies(_v(0, 0, 0)), isFalse);
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 1, 0),
          minimumClientVersion: _v(0, 0, 0),
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.ok,
      );
    });

    test('client too old when below the server-advertised floor', () {
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 1, 0),
          minimumClientVersion: _v(9, 9, 9),
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.clientTooOld,
      );
    });

    test('client satisfies an equal floor', () {
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 1, 0),
          minimumClientVersion: _v(0, 0, 0),
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.ok,
      );
    });

    test('serverTooOld wins when both are outdated', () {
      expect(
        checkServerCompatibility(
          serverVersion: _v(0, 0, 1),
          minimumClientVersion: _v(9, 9, 9),
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.serverTooOld,
      );
    });

    test('missing server version never blocks', () {
      expect(
        checkServerCompatibility(
          serverVersion: null,
          minimumClientVersion: _v(9, 9, 9),
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.clientTooOld,
      );
      expect(
        checkServerCompatibility(
          serverVersion: null,
          minimumClientVersion: null,
          clientVersion: 'v0.0.0.beta',
        ),
        ServerCompatibility.ok,
      );
    });
  });
}
