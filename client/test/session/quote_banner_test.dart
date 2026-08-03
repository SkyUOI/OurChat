import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session/session_tab.dart';
import 'package:ourchat/session/state.dart';
import 'test_harness.dart';

void main() {
  ProviderContainer makeContainer() {
    final c = ProviderContainer(overrides: [
      activeAccountTestOverride,
      ourChatServerProvider.overrideWithValue(
        FakeOurChatServer(MockOurChatClient()),
      ),
      overrideAccount(Int64(2), buildTestAccount(Int64(2), 'bob', displayName: 'Bob')),
    ]);
    addTearDown(c.dispose);
    return c;
  }

  testWidgets('shows the quote banner above the input when a target is set',
      (tester) async {
    final container = makeContainer();
    container.read(quoteTargetProvider.notifier).setQuote(
      UserMsg(senderId: Int64(2), eventId: Int64(5), markdownText: 'quote me'),
    );

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    expect(find.text('Bob: quote me'), findsOneWidget);
    await tester.pumpAndSettle();
  });

  testWidgets('close button clears the quote target and hides the banner',
      (tester) async {
    final container = makeContainer();
    container.read(quoteTargetProvider.notifier).setQuote(
      UserMsg(senderId: Int64(2), eventId: Int64(5), markdownText: 'quote me'),
    );

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();
    expect(find.text('Bob: quote me'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.close));
    await tester.pump();
    await tester.pumpAndSettle();

    expect(container.read(quoteTargetProvider), isNull);
    expect(find.text('Bob: quote me'), findsNothing);
  });

  testWidgets('no banner rendered when no quote target is set', (tester) async {
    final container = makeContainer();

    await tester.pumpWidget(
      buildTestApp(container: container, child: const SessionTab()),
    );
    await tester.pump();

    expect(find.byIcon(Icons.format_quote), findsNothing);
  });
}
