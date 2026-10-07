import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/config.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';

/// Regression tests for the app bootstrap wiring (found by manually testing
/// the Flutter web build): `main()` loads the persisted config and hands it
/// to the provider tree via [primeConfig]. Before that existed, the provider
/// built from defaults with `prefsWithCache == null`, so every
/// `saveConfig()` from provider mutators was a silent no-op — added servers,
/// saved accounts and settings were lost on every restart (on web and
/// desktop alike).
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('primeConfig wires the provider, and provider writes persist', () async {
    // One in-memory platform store shared by both "launches" below, standing
    // in for the browser's localStorage / the desktop prefs file.
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.empty();

    // ── First launch (mirrors main()): nothing stored → defaults persisted,
    //    then primed into the provider tree. ──
    final prefs1 = await SharedPreferencesWithCache.create(
      cacheOptions: const SharedPreferencesWithCacheOptions(),
    );
    var config = OurChatConfig.defaults.copyWith(prefsWithCache: prefs1);
    config.saveConfig();
    primeConfig(config);

    final container1 = ProviderContainer();
    addTearDown(container1.dispose);
    container1
        .read(configProvider.notifier)
        .upsertServer(
          ServerConfig(host: 'localhost', port: 7777, label: 'dev'),
        );

    final stored =
        jsonDecode(prefs1.getString('config')!) as Map<String, dynamic>;
    expect(
      (stored['servers'] as List).map((s) => '${s['host']}:${s['port']}'),
      contains('localhost:7777'),
      reason: 'the provider write must reach the persisted blob',
    );

    // ── Restart: reload from prefs and re-prime, as main() does. ──
    final prefs2 = await SharedPreferencesWithCache.create(
      cacheOptions: const SharedPreferencesWithCacheOptions(),
    );
    final loaded = OurChatConfig.fromJson(
      jsonDecode(prefs2.getString('config')!) as Map<String, dynamic>,
    );
    final config2 = loaded.copyWith(prefsWithCache: prefs2);
    primeConfig(config2);

    final container2 = ProviderContainer();
    addTearDown(container2.dispose);
    final restored = container2.read(configProvider);
    expect(
      restored.servers.map((s) => '${s.host}:${s.port}'),
      contains('localhost:7777'),
      reason: 'the added server must survive the restart',
    );
    expect(
      restored.prefsWithCache,
      same(prefs2),
      reason:
          'the restored provider state must carry the prefs handle; '
          'a null handle makes every provider saveConfig() a no-op',
    );

    // Writes through the restored provider persist too.
    container2.read(configProvider.notifier).setColor(0xFF112233);
    final stored2 =
        jsonDecode(prefs2.getString('config')!) as Map<String, dynamic>;
    expect(stored2['color'], 0xFF112233);
  });

  test('without primeConfig the provider falls back to defaults', () {
    // Documents the bootstrap contract: an app run that skips primeConfig
    // (the pre-fix state) gets a defaults-only config with no prefs handle.
    primeConfig(OurChatConfig.defaults);
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final config = container.read(configProvider);
    expect(config.servers, OurChatConfig.defaults.servers);
    expect(config.prefsWithCache, isNull);
  });
}
