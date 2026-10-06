import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/session_record.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'test_harness.dart';

void main() {
  ProviderContainer makeContainer() {
    final c = ProviderContainer(
      overrides: [
        activeAccountTestOverride,
        ourChatServerProvider.overrideWithValue(
          FakeOurChatServer(MockOurChatClient()),
        ),
        overrideAccount(Int64(1), buildTestAccount(Int64(1), 'alice')),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  Future<void> openSession(ProviderContainer container, Int64 sessionId) async {
    // Mimic what the mobile back button / session list does: leaving the tab
    // clears it, re-entering opens a fresh one.
    container.read(sessionProvider.notifier).clearTab();
    container
        .read(sessionProvider.notifier)
        .openSessionTab(sessionId, 'session $sessionId');
  }

  testWidgets(
    'leaving and re-entering a session clears the draft and preview bubble',
    (tester) async {
      final container = makeContainer();
      container.read(sessionProvider.notifier).state = SessionState(
        tabIndex: TabType.session,
        currentSessionId: Int64(1),
        currentSessionRecords: const [],
      );

      await tester.pumpWidget(
        buildTestApp(container: container, child: const SessionTab()),
      );
      await tester.pump();

      await tester.enterText(find.byType(TextFormField), 'draft text');
      await tester.pump();

      // The draft is stored and the translucent preview bubble is visible.
      expect(container.read(inputTextProvider), 'draft text');
      expect(find.byType(MessageWidget), findsOneWidget);
      // Matches both the input field and the (selectable) preview bubble.
      expect(find.text('draft text'), findsWidgets);

      await openSession(container, Int64(2));
      await tester.pump();

      expect(container.read(inputTextProvider), '');
      final field = tester.widget<TextFormField>(find.byType(TextFormField));
      expect(field.controller!.text, '');
      // No preview bubble residue either.
      expect(find.byType(MessageWidget), findsNothing);
      expect(find.text('draft text'), findsNothing);
      // Flush the one-shot timer the input area schedules (pre-existing
      // behavior of the session input widgets).
      await tester.pump(const Duration(seconds: 10));
    },
  );

  testWidgets('switching sessions resets the previous draft', (tester) async {
    final container = makeContainer();
    container
        .read(sessionProvider.notifier)
        .openSessionTab(Int64(1), 'session 1');

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextFormField), 'half-written');
    await tester.pump();
    expect(container.read(inputTextProvider), 'half-written');

    // Desktop-style switch: same SessionTab widget, different session.
    container
        .read(sessionProvider.notifier)
        .openSessionTab(Int64(2), 'session 2');
    await tester.pump();

    expect(container.read(inputTextProvider), '');
    final field = tester.widget<TextFormField>(find.byType(TextFormField));
    expect(field.controller!.text, '');
    expect(find.byType(MessageWidget), findsNothing);
    expect(find.text('half-written'), findsNothing);
    await tester.pump(const Duration(seconds: 10));
  });

  testWidgets('disposes the text controller with the state', (tester) async {
    final container = makeContainer();
    container
        .read(sessionProvider.notifier)
        .openSessionTab(Int64(1), 'session 1');

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    final field = tester.widget<TextFormField>(find.byType(TextFormField));
    final controller = field.controller!;
    expect(controller.text, isEmpty);

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SizedBox()),
    );
    await tester.pump(const Duration(seconds: 10));

    // Using the controller after disposal must throw — proving dispose ran.
    expect(() => controller.text = 'x', throwsFlutterError);
  });
}
