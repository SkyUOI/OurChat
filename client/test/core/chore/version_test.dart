import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore/version.dart';

/// Tests for `analyzeVersionString`. This is the release-selection logic used
/// by the OTA updater, so a bug here means either missing updates or
/// downgrading users.
///
/// Return shape: `[major, minor, patch, prereleaseString, isAlpha, isBeta]`.
void main() {
  group('analyzeVersionString', () {
    test('stable version parses x.y.z', () {
      final r = analyzeVersionString('v1.2.3');
      expect(r[0], 1);
      expect(r[1], 2);
      expect(r[2], 3);
      expect(r[4], isFalse, reason: 'not alpha');
      expect(r[5], isFalse, reason: 'not beta');
    });

    test('beta prerelease is flagged', () {
      final r = analyzeVersionString('v2.0.0-beta');
      expect(r[0], 2);
      expect(r[1], 0);
      expect(r[2], 0);
      expect(r[5], isTrue, reason: 'beta detected');
      expect(r[4], isFalse, reason: 'not alpha');
    });

    test('alpha prerelease is flagged', () {
      final r = analyzeVersionString('v0.9.0-alpha1');
      expect(r[0], 0);
      expect(r[1], 9);
      expect(r[2], 0);
      expect(r[4], isTrue, reason: 'alpha detected');
    });

    test('numeric comparison drives update decisions', () {
      // Mirrors the loop in needUpdate(): for each component, the first
      // difference decides newer/older.
      bool isNewer(String latest, String current) {
        final l = analyzeVersionString(latest);
        final c = analyzeVersionString(current);
        for (var j = 0; j < 3; j++) {
          if (l[j] > c[j]) return true;
          if (l[j] < c[j]) return false;
        }
        return false;
      }

      expect(isNewer('v1.2.3', 'v1.2.2'), isTrue);
      expect(isNewer('v1.3.0', 'v1.2.9'), isTrue);
      expect(isNewer('v2.0.0', 'v1.99.99'), isTrue);
      expect(isNewer('v1.2.2', 'v1.2.3'), isFalse);
      expect(isNewer('v1.2.3', 'v1.2.3'), isFalse);
    });

    test('handles versions without leading v', () {
      final r = analyzeVersionString('3.1.4');
      expect(r[0], 3);
      expect(r[1], 1);
      expect(r[2], 4);
    });
  });
}
