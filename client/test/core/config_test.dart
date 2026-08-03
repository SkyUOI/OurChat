import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/config.dart';

/// Tests for `OurChatConfig` serialization.
///
/// Security invariant: credentials are NEVER stored in this config blob —
/// passwords live only in `SecretStore` (platform-backed secure storage).
/// `SavedAccount` carries only non-secret identity metadata (ocid, email,
/// avatar key), never the password.
void main() {
  group('OurChatConfig serialization', () {
    test('savedAccounts round trip', () {
      final original = OurChatConfig(
        savedAccounts: [
          SavedAccount(
            serverId: 'srv-a',
            accountId: 42,
            ocid: 'oc_alice',
            email: 'alice@example.com',
            avatarKey: 'k1',
            lastLoginAt: DateTime.utc(2024, 1, 2, 3, 4, 5),
          ),
          SavedAccount(
            serverId: 'srv-b',
            accountId: 42, // same numeric id, different server — must not collide
            lastLoginAt: DateTime.utc(2024, 6, 7),
          ),
        ],
      );
      final restored = OurChatConfig.fromJson(
        jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
      );
      expect(restored.savedAccounts.length, 2);
      expect(restored.savedAccounts[0].serverId, 'srv-a');
      expect(restored.savedAccounts[0].accountId, 42);
      expect(restored.savedAccounts[0].email, 'alice@example.com');
      expect(restored.savedAccounts[1].serverId, 'srv-b');
      expect(restored.savedAccounts[1].accountId, 42);
    });

    test('active account pointer round trip', () {
      final original = OurChatConfig(
        activeServerId: 'srv-a',
        activeAccountId: 7,
      );
      final restored = OurChatConfig.fromJson(
        jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
      );
      expect(restored.activeServerId, 'srv-a');
      expect(restored.activeAccountId, 7);
    });

    test('no password field is ever present in the serialized blob', () {
      final cfg = OurChatConfig(
        savedAccounts: [
          SavedAccount(
            serverId: 'srv',
            accountId: 1,
            lastLoginAt: DateTime.utc(2024),
          ),
        ],
      );
      final json = jsonEncode(cfg.toJson());
      expect(json, isNot(contains('password')));
      expect(json, isNot(contains('Password')));
      expect(json, isNot(contains('credential')));
    });

    test('ServerConfig carries uniqueIdentifier + label + isTLS', () {
      final original = OurChatConfig(
        servers: [
          ServerConfig(
            host: 'example.com',
            port: 7777,
            uniqueIdentifier: 'srv-uuid-1',
            label: 'My Server',
            isTLS: true,
          ),
        ],
      );
      final restored = OurChatConfig.fromJson(
        jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
      );
      expect(restored.servers.length, 1);
      expect(restored.servers[0].host, 'example.com');
      expect(restored.servers[0].port, 7777);
      expect(restored.servers[0].uniqueIdentifier, 'srv-uuid-1');
      expect(restored.servers[0].label, 'My Server');
      expect(restored.servers[0].isTLS, isTrue);
    });

    test('defaults are sensible', () {
      final d = OurChatConfig.defaults;
      expect(d.servers, isNotEmpty);
      expect(d.color, isPositive);
      expect(d.logLevel, 'info');
      expect(d.savedAccounts, isEmpty);
      expect(d.activeServerId, isNull);
      expect(d.activeAccountId, isNull);
      expect(d.updateSource, startsWith('https://'));
    });

    test('ConfigNotifier.upsertSavedAccount updates in place by key', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(configProvider.notifier);
      notifier.init(OurChatConfig());
      notifier.upsertSavedAccount(
        SavedAccount(
          serverId: 's1',
          accountId: 1,
          lastLoginAt: DateTime.utc(2024),
        ),
      );
      notifier.upsertSavedAccount(
        SavedAccount(
          serverId: 's1',
          accountId: 1,
          email: 'updated@example.com',
          lastLoginAt: DateTime.utc(2025),
        ),
      );
      expect(container.read(configProvider).savedAccounts.length, 1);
      expect(
        container.read(configProvider).savedAccounts.single.email,
        'updated@example.com',
      );
    });
  });
}
