import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/main.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';

/// Tests for the configurable close-button behavior (issue #203) and the
/// notification privacy switch persistence (issue #199): defaults, JSON round
/// trips, ConfigNotifier persistence against an in-memory prefs store, and
/// the pure window-close decision function.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('defaults', () {
    test('close button minimizes to tray and notifications show content', () {
      final d = OurChatConfig.defaults;
      expect(d.closeBehavior, CloseBehavior.minimizeToTray);
      expect(d.notificationShowMessageContent, isTrue);
    });
  });

  group('serialization', () {
    test('closeBehavior and notificationShowMessageContent round trip', () {
      final original = OurChatConfig(
        closeBehavior: CloseBehavior.exit,
        notificationShowMessageContent: false,
      );
      final restored = OurChatConfig.fromJson(
        jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
      );
      expect(restored.closeBehavior, CloseBehavior.exit);
      expect(restored.notificationShowMessageContent, isFalse);
    });

    test(
      'legacy config JSON without the new fields falls back to defaults',
      () {
        final restored = OurChatConfig.fromJson({});
        expect(restored.closeBehavior, CloseBehavior.minimizeToTray);
        expect(restored.notificationShowMessageContent, isTrue);
      },
    );
  });

  group('ConfigNotifier persistence (in-memory prefs store)', () {
    setUp(() async {
      // No platform store exists in the test VM; swap in a fresh in-memory
      // one per test (there is no previous instance to restore).
      SharedPreferencesAsyncPlatform.instance =
          InMemorySharedPreferencesAsync.empty();
    });

    tearDown(() {
      SharedPreferencesAsyncPlatform.instance = null;
    });

    test('setCloseBehavior(exit) persists and survives a reload', () async {
      final prefs = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions(),
      );
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(configProvider.notifier);
      notifier.init(OurChatConfig.defaults.copyWith(prefsWithCache: prefs));

      notifier.setCloseBehavior(CloseBehavior.exit);

      // Persisted blob contains the chosen behavior.
      final stored =
          jsonDecode(prefs.getString('config')!) as Map<String, dynamic>;
      expect(stored['closeBehavior'], 'exit');

      // A reload from the same storage restores it into the state.
      notifier.reload();
      expect(container.read(configProvider).closeBehavior, CloseBehavior.exit);
    });

    test('setNotificationShowMessageContent(false) persists', () async {
      final prefs = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions(),
      );
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(configProvider.notifier);
      notifier.init(OurChatConfig.defaults.copyWith(prefsWithCache: prefs));

      notifier.setNotificationShowMessageContent(false);

      final stored =
          jsonDecode(prefs.getString('config')!) as Map<String, dynamic>;
      expect(stored['notificationShowMessageContent'], isFalse);
      notifier.reload();
      expect(
        container.read(configProvider).notificationShowMessageContent,
        isFalse,
      );
    });

    test('reset() restores the defaults', () async {
      final prefs = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions(),
      );
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(configProvider.notifier);
      notifier.init(OurChatConfig.defaults.copyWith(prefsWithCache: prefs));

      notifier.setCloseBehavior(CloseBehavior.exit);
      notifier.setNotificationShowMessageContent(false);
      notifier.reset();

      expect(
        container.read(configProvider).closeBehavior,
        CloseBehavior.minimizeToTray,
      );
      expect(
        container.read(configProvider).notificationShowMessageContent,
        isTrue,
      );
    });
  });

  group('resolveWindowCloseAction (onWindowClose decision)', () {
    test('minimizeToTray setting hides the window', () {
      expect(
        resolveWindowCloseAction(CloseBehavior.minimizeToTray),
        CloseAction.minimizeToTray,
      );
    });

    test('exit setting quits the app', () {
      expect(resolveWindowCloseAction(CloseBehavior.exit), CloseAction.exitApp);
    });
  });
}
