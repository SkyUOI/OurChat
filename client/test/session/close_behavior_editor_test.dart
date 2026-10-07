import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/setting.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';

import '../session/test_harness.dart';

/// The settings editor for the close-button behavior (issue #203): switching
/// the dropdown persists the choice through the real ConfigNotifier against
/// an in-memory preferences store.
void main() {
  late SharedPreferencesWithCache prefs;

  setUp(() async {
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.empty();
    prefs = await SharedPreferencesWithCache.create(
      cacheOptions: const SharedPreferencesWithCacheOptions(),
    );
  });

  tearDown(() {
    SharedPreferencesAsyncPlatform.instance = null;
  });

  Future<ProviderContainer> pumpEditor(WidgetTester tester) async {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final notifier = container.read(configProvider.notifier);
    notifier.init(OurChatConfig.defaults.copyWith(prefsWithCache: prefs));
    expect(
      container.read(configProvider).closeBehavior,
      CloseBehavior.minimizeToTray,
    );

    await tester.pumpWidget(
      buildTestApp(container: container, child: const CloseBehaviorEditor()),
    );
    await tester.pump();
    return container;
  }

  testWidgets('defaults to minimize-to-tray and persists "Exit"', (
    tester,
  ) async {
    final container = await pumpEditor(tester);

    expect(find.text(l10n.closeBehaviorMinimizeToTray), findsOneWidget);

    await tester.tap(find.byType(DropdownButtonFormField<CloseBehavior>));
    await tester.pumpAndSettle();
    await tester.tap(find.text(l10n.closeBehaviorExit).last);
    await tester.pumpAndSettle();

    expect(container.read(configProvider).closeBehavior, CloseBehavior.exit);
    final stored =
        jsonDecode(prefs.getString('config')!) as Map<String, dynamic>;
    expect(stored['closeBehavior'], 'exit');
  });
}
