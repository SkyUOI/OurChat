import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/config.dart';

/// Tests for `OurChatConfig` serialization.
///
/// The most important property under test is a SECURITY invariant (C-4): the
/// saved login password (`recentPassword`) must NEVER be written to the JSON
/// blob that is persisted via SharedPreferences. Persisting happens through
/// `saveConfig()` -> `jsonEncode(toJson())`. If `recentPassword` ever leaks
/// back into the serialized form, every saved login would store the password in
/// plaintext on disk again.
void main() {
  group('OurChatConfig serialization', () {
    test('recentPassword is excluded from toJson (C-4 security invariant)', () {
      final cfg = OurChatConfig(
        recentAccount: 'alice@example.com',
        recentPassword: 'super-secret-123',
      );
      final json = jsonEncode(cfg.toJson());

      // The plaintext password value must not appear anywhere in the blob.
      expect(json, isNot(contains('super-secret-123')));
      // The key must not appear either (so even an empty password is not
      // serialized as a field).
      expect(json, isNot(contains('recentPassword')));
      expect(json, isNot(contains('recent_password')));
    });

    test('recentAccount (non-secret) IS persisted', () {
      final cfg = OurChatConfig(recentAccount: 'alice@example.com');
      final json = jsonEncode(cfg.toJson());
      expect(json, contains('alice@example.com'));
      expect(json, contains('recentAccount'));
    });

    test('fromJson ignores any recentPassword present in the blob', () {
      // Even if an old/legacy blob (or a tampered one) carries a password, the
      // deserialized config must not pick it up.
      final legacy = {
        'recentAccount': 'bob',
        'recentPassword': 'legacy-leak',
        'color': 0xFF2196F3,
      };
      final cfg = OurChatConfig.fromJson(legacy);
      expect(cfg.recentAccount, 'bob');
      expect(cfg.recentPassword, '',
          reason: 'recentPassword must always deserialize to its default');
    });

    test('round trip preserves non-sensitive fields', () {
      final original = OurChatConfig(
        servers: [
          ServerConfig(host: 'example.com', port: 7777),
          ServerConfig(host: '1.2.3.4', port: 9999),
        ],
        color: 0xFF112233,
        logLevel: 'warning',
        recentAccount: 'alice',
        recentAvatarUrl: 'http://example.com/a.png',
        updateSource: 'https://example.com/releases',
        language: const LanguageConfig(
          languageCode: 'zh',
          scriptCode: 'Hans',
          countryCode: 'CN',
        ),
      );
      final restored = OurChatConfig.fromJson(
        jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
      );
      expect(restored.servers.length, 2);
      expect(restored.servers[0].host, 'example.com');
      expect(restored.servers[0].port, 7777);
      expect(restored.servers[1].port, 9999);
      expect(restored.color, 0xFF112233);
      expect(restored.logLevel, 'warning');
      expect(restored.recentAccount, 'alice');
      expect(restored.recentAvatarUrl, 'http://example.com/a.png');
      expect(restored.updateSource, 'https://example.com/releases');
      expect(restored.language?.languageCode, 'zh');
      expect(restored.language?.countryCode, 'CN');
      // And the password did not survive the round trip.
      expect(restored.recentPassword, '');
    });

    test('defaults are sensible', () {
      final d = OurChatConfig.defaults;
      expect(d.servers, isNotEmpty);
      expect(d.color, isPositive);
      expect(d.logLevel, 'info');
      expect(d.recentPassword, '');
      expect(d.updateSource, startsWith('https://'));
    });
  });
}
