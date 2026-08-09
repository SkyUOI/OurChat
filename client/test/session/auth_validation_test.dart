import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/auth.dart';
import 'package:ourchat/main.dart';

import 'test_harness.dart';

/// The OurChat-style dark bottom snackbar used for auth errors.
final Color kAuthSnackBarColor = Colors.black87;

void main() {
  testWidgets('login with empty fields is blocked without a snackbar', (
    tester,
  ) async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    await tester.pumpWidget(
      buildTestApp(container: container, child: const Login()),
    );

    await tester.tap(find.byIcon(Icons.login));
    await tester.pump();

    // Empty fields are blocked via inline form validation only — no snackbar.
    expect(find.byType(SnackBar), findsNothing);
    expect(find.text(l10n.cantBeEmpty), findsWidgets);
  });

  testWidgets('register with empty fields is blocked without a snackbar', (
    tester,
  ) async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    await tester.pumpWidget(
      buildTestApp(container: container, child: const Register()),
    );

    await tester.tap(find.byIcon(Icons.app_registration));
    await tester.pump();

    expect(find.byType(SnackBar), findsNothing);
    expect(find.text(l10n.cantBeEmpty), findsWidgets);
  });

  testWidgets('login failure surfaces the auth error in the dark snackbar', (
    tester,
  ) async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    await tester.pumpWidget(
      buildTestApp(container: container, child: const Login()),
    );

    final fields = find.byType(TextFormField);
    await tester.enterText(fields.at(0), 'user@test.local');
    await tester.enterText(fields.at(1), 'password123');
    await tester.tap(find.byIcon(Icons.login));
    await tester.pump();
    await tester.pump();

    // The default server has no uniqueIdentifier yet, so login fails with
    // l10n.serverNotIdentified — surfaced via the dark snackbar.
    final snackBar = tester.widget<SnackBar>(find.byType(SnackBar));
    expect(snackBar.backgroundColor, kAuthSnackBarColor);
    expect(find.text(l10n.serverNotIdentified), findsOneWidget);
  });
}
