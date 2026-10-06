import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/announcement_page.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/l10n/app_localizations.dart';

class Setting extends StatelessWidget {
  const Setting({super.key});

  @override
  Widget build(BuildContext context) {
    final formKey = GlobalKey<FormState>();
    return Form(
      key: formKey,
      child: Center(
        child: Padding(
          padding: const EdgeInsets.only(top: 20.0, bottom: 20.0),
          child: Column(
            children: [
              Expanded(
                child: SingleChildScrollView(
                  // Scrollable
                  child: Column(
                    children: [
                      ListTile(
                        leading: const Icon(Icons.campaign),
                        title: Text(l10n.announcement),
                        onTap: () {
                          Navigator.push(
                            context,
                            MaterialPageRoute(
                              builder: (context) =>
                                  const AnnouncementListPage(),
                            ),
                          );
                        },
                      ),
                      SeedColorEditor(),
                      LogLevelSelector(),
                      DisplayModeEditor(),
                      NotificationContentEditor(),
                      CloseBehaviorEditor(),
                      LanguageEditor(),
                      if (enableVersionCheck) UpdateSourceEditor(),
                    ],
                  ),
                ),
              ),
              DialogButtons(formKey: formKey), // Confirm/Reset
            ],
          ),
        ),
      ),
    );
  }
}

class UpdateSourceEditor extends ConsumerStatefulWidget {
  const UpdateSourceEditor({super.key});

  @override
  ConsumerState<UpdateSourceEditor> createState() => _UpdateSourceEditorState();
}

class _UpdateSourceEditorState extends ConsumerState<UpdateSourceEditor> {
  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configProvider);
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(height: 30.0, width: 30.0, child: Icon(Icons.link)),
        ),
        Expanded(
          child: TextFormField(
            initialValue: config.updateSource,
            decoration: InputDecoration(label: Text("URL")),
            autovalidateMode: AutovalidateMode.onUnfocus,
            validator: (value) {
              ref.read(configProvider.notifier).setUpdateSource(value!);
              return null;
            },
          ),
        ),
      ],
    );
  }
}

class LanguageEditor extends ConsumerWidget {
  const LanguageEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    var config = ref.watch(configProvider);
    var language = config.language ?? LanguageConfig.defaults;
    List<DropdownMenuItem> languages = [];
    for (int i = 0; i < AppLocalizations.supportedLocales.length; i++) {
      languages.add(
        DropdownMenuItem(
          value:
              "${AppLocalizations.supportedLocales.elementAt(i).languageCode}-${AppLocalizations.supportedLocales.elementAt(i).scriptCode}-${AppLocalizations.supportedLocales.elementAt(i).countryCode}",
          child: Text(
            AppLocalizations.supportedLocales.elementAt(i).languageCode,
          ),
        ),
      );
    }
    // Find matching item by language code, not exact string
    String initialValue =
        languages
                .firstWhere(
                  (item) =>
                      (item.value as String).startsWith(language.languageCode),
                  orElse: () => languages.first,
                )
                .value
            as String;
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: Icon(Icons.translate),
          ),
        ),
        Expanded(
          child: DropdownButtonFormField(
            decoration: InputDecoration(label: Text(l10n.language)),
            initialValue: initialValue,
            items: languages,
            onChanged: (value) {
              List languageStringData = value.split("-");
              List languageData = [];
              for (int i = 0; i < languageStringData.length; i++) {
                if (languageStringData[i] == "null") {
                  languageData.add(null);
                } else {
                  languageData.add(languageStringData[i]);
                }
              }
              ref
                  .read(configProvider.notifier)
                  .setLanguage(
                    LanguageConfig(
                      languageCode: languageData[0] ?? '',
                      scriptCode: languageData[1] ?? '',
                      countryCode: languageData[2] ?? '',
                    ),
                  );

              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Localizations.override(
                    context: context,
                    locale: Locale.fromSubtags(
                      languageCode: languageData[0]!,
                      scriptCode: languageData[1],
                      countryCode: languageData[2],
                    ),
                    child: Builder(
                      builder: (context) {
                        return Text(AppLocalizations.of(context)!.needRestart);
                      },
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

class SeedColorEditor extends ConsumerWidget {
  const SeedColorEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    var config = ref.watch(configProvider);
    var seedColor = config.color;
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: Container(
              decoration: BoxDecoration(
                border: Border.all(
                  color: ColorScheme.fromSeed(
                    seedColor: Color(seedColor),
                  ).secondary,
                ),
                color: Color(config.color),
              ),
            ),
          ),
        ),
        Expanded(
          child: TextFormField(
            decoration: InputDecoration(labelText: l10n.themeColorSeed),
            controller: TextEditingController(
              text: "0x${config.color.toRadixString(16)}",
            ),
            autovalidateMode: AutovalidateMode.onUnfocus,
            validator: (value) {
              if (value == null || value.isEmpty) {
                return AppLocalizations.of(context)!.cantBeEmpty;
              }
              int colorCode;
              try {
                colorCode = int.parse(value);
              } catch (e) {
                return AppLocalizations.of(context)!.invalidColorCode;
              }
              ref.read(configProvider.notifier).setColor(colorCode);
              return null;
            },
          ),
        ),
      ],
    );
  }
}

class DisplayModeEditor extends ConsumerWidget {
  const DisplayModeEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mode = ref.watch(configProvider).displayMode;
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(width: 30.0, height: 30.0, child: Icon(Icons.inbox)),
        ),
        Expanded(
          child: DropdownButtonFormField<UiDisplayMode>(
            decoration: InputDecoration(label: Text(l10n.displayMode)),
            initialValue: mode,
            items: [
              DropdownMenuItem(
                value: UiDisplayMode.accountSwitcher,
                child: Text(l10n.accountSwitch),
              ),
              DropdownMenuItem(
                value: UiDisplayMode.unifiedInbox,
                child: Text(l10n.unifiedInbox),
              ),
            ],
            onChanged: (v) {
              if (v != null) {
                ref.read(configProvider.notifier).setDisplayMode(v);
              }
            },
          ),
        ),
      ],
    );
  }
}

/// Privacy switch for new-message notifications (issue #199): when off, the
/// notification body is always a generic "New message" instead of the message
/// text.
class NotificationContentEditor extends ConsumerWidget {
  const NotificationContentEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final showContent = ref.watch(configProvider).notificationShowMessageContent;
    return Row(
      children: [
        const Padding(
          padding: EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: Icon(Icons.notifications),
          ),
        ),
        Expanded(
          child: SwitchListTile(
            title: Text(l10n.notificationShowMessageContent),
            value: showContent,
            onChanged: (v) {
              ref
                  .read(configProvider.notifier)
                  .setNotificationShowMessageContent(v);
            },
          ),
        ),
      ],
    );
  }
}

/// Editor for what the desktop window close (X) button does (issue #203):
/// minimize to the system tray (default) or exit the application.
class CloseBehaviorEditor extends ConsumerWidget {
  const CloseBehaviorEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final behavior = ref.watch(configProvider).closeBehavior;
    return Row(
      children: [
        const Padding(
          padding: EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: Icon(Icons.close),
          ),
        ),
        Expanded(
          child: DropdownButtonFormField<CloseBehavior>(
            decoration: InputDecoration(label: Text(l10n.closeBehavior)),
            initialValue: behavior,
            items: [
              DropdownMenuItem(
                value: CloseBehavior.minimizeToTray,
                child: Text(l10n.closeBehaviorMinimizeToTray),
              ),
              DropdownMenuItem(
                value: CloseBehavior.exit,
                child: Text(l10n.closeBehaviorExit),
              ),
            ],
            onChanged: (v) {
              if (v != null) {
                ref.read(configProvider.notifier).setCloseBehavior(v);
              }
            },
          ),
        ),
      ],
    );
  }
}

class DialogButtons extends ConsumerWidget {
  const DialogButtons({super.key, required this.formKey});

  final GlobalKey<FormState> formKey;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Padding(
      padding: EdgeInsets.all(AppStyles.mediumPadding),
      child: ElevatedButton.icon(
        style: AppStyles.defaultButtonStyle,
        icon: Icon(Icons.refresh),
        onPressed: () {
          ref.read(configProvider.notifier).reset();
        },
        label: Text(l10n.reset),
      ),
    );
  }
}

class LogLevelSelector extends ConsumerWidget {
  const LogLevelSelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    var config = ref.watch(configProvider);
    List<DropdownMenuItem> dropDownItems = [];
    for (var i = 0; i < logLevels.length; i++) {
      var value = logLevels[i];
      dropDownItems.add(
        DropdownMenuItem(
          value: value,
          child: Row(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [getLevelIcon(value), Text(value)],
          ),
        ),
      );
    }
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.defaultPadding),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: getLevelIcon(config.logLevel),
          ),
        ),
        Expanded(
          child: DropdownButtonFormField(
            decoration: InputDecoration(label: Text(l10n.logLevel)),
            initialValue: config.logLevel,
            items: dropDownItems,
            selectedItemBuilder: (context) {
              List<DropdownMenuItem> selectedItems = [];
              for (var i = 0; i < logLevels.length; i++) {
                var value = logLevels[i];
                selectedItems.add(
                  DropdownMenuItem(value: value, child: Text(value)),
                );
              }
              return selectedItems;
            },
            onChanged: (value) {
              ref.read(configProvider.notifier).setLogLevel(value);
            },
          ),
        ),
      ],
    );
  }

  /// Returns an Icon widget representing the log level.
  ///
  /// This function maps a given log level string to a corresponding
  /// Icon widget with a specific color and size. The available log
  /// levels and their corresponding icons are:
  /// - 'debug': A green bug report icon.
  /// - 'info': A blue info icon.
  /// - 'warning': An orange warning icon.
  /// - 'error': A red error icon.
  /// - 'fatal': A purple dangerous icon.
  /// - 'trace': A grey code icon.
  ///
  /// If the log level does not match any of the predefined cases,
  /// a default help icon is returned.
  Icon getLevelIcon(String level) {
    var size = 35.0;
    switch (level) {
      case 'debug':
        return Icon(Icons.bug_report, size: size);
      case 'info':
        return Icon(Icons.info, size: size);
      case 'warning':
        return Icon(Icons.warning, size: size);
      case 'error':
        return Icon(Icons.error, size: size);
      case 'fatal':
        return Icon(Icons.dangerous, size: size);
      case 'trace':
        return Icon(Icons.code, size: size);
      default:
        return Icon(Icons.help);
    }
  }
}
