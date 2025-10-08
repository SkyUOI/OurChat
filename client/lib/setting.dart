import 'package:flutter/material.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/main.dart';
import 'package:provider/provider.dart';
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
                  // 可滚动
                  child: Column(
                    children: [
                      SeedColorEditor(),
                      LogLevelSelector(),
                      LanguageEditor()
                    ],
                  ),
                ),
              ),
              DialogButtons(formKey: formKey) // 确定/重置
            ],
          ),
        ),
      ),
    );
  }
}

class LanguageEditor extends StatelessWidget {
  const LanguageEditor({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    var language = ourchatAppState.config["language"];
    List<DropdownMenuItem> languages = [];
    for (int i = 0; i < AppLocalizations.supportedLocales.length; i++) {
      languages.add(DropdownMenuItem(
          value:
              "${AppLocalizations.supportedLocales.elementAt(i).languageCode}-${AppLocalizations.supportedLocales.elementAt(i).scriptCode}-${AppLocalizations.supportedLocales.elementAt(i).countryCode}",
          child: Text(
              AppLocalizations.supportedLocales.elementAt(i).languageCode)));
    }
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(5.0),
          child: SizedBox(
            width: 30.0,
            height: 30.0,
            child: Icon(Icons.translate),
          ),
        ),
        Expanded(
          child: DropdownButtonFormField(
            decoration: InputDecoration(label: Text(l10n.language)),
            initialValue: "${language[0]}-${language[1]}-${language[2]}",
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
              ourchatAppState.config["language"] = languageData;

              ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                  content: Localizations.override(
                      context: context,
                      locale: Locale.fromSubtags(
                          languageCode: languageData[0]!,
                          scriptCode: languageData[1],
                          countryCode: languageData[2]),
                      child: Builder(builder: (context) {
                        return Text(AppLocalizations.of(context)!.needRestart);
                      }))));
            },
          ),
        ),
      ],
    );
  }
}

class SeedColorEditor extends StatelessWidget {
  const SeedColorEditor({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    var seedColor = ourchatAppState.config["color"];
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(5.0),
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
                color: Color(
                  ourchatAppState.config["color"],
                ),
              ),
            ),
          ),
        ),
        Expanded(
          child: TextFormField(
              decoration: InputDecoration(
                labelText: l10n.themeColorSeed,
              ),
              controller: TextEditingController(
                text: "0x${ourchatAppState.config["color"].toRadixString(16)}",
              ),
              autovalidateMode: AutovalidateMode.onUnfocus,
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return AppLocalizations.of(
                    context,
                  )!
                      .cantBeEmpty;
                }
                int colorCode;
                try {
                  colorCode = int.parse(
                    value,
                  );
                } catch (e) {
                  return AppLocalizations.of(
                    context,
                  )!
                      .invalidColorCode;
                }
                ourchatAppState.config["color"] = colorCode;
                return null;
              }),
        ),
      ],
    );
  }
}

class DialogButtons extends StatelessWidget {
  const DialogButtons({
    super.key,
    required this.formKey,
  });

  final GlobalKey<FormState> formKey;

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    return Padding(
      padding: EdgeInsets.all(AppStyles.mediumPadding),
      child: ElevatedButton.icon(
        style: AppStyles.defaultButtonStyle,
        icon: Icon(Icons.refresh),
        onPressed: () {
          var defaultConfig = appState.config.getDefaultConfig();
          List keys = ["color", "log_level"];
          for (String key in keys) {
            appState.config.data[key] = defaultConfig[key];
          }
          appState.update();
        },
        label: Text(l10n.reset),
      ),
    );
  }
}

class LogLevelSelector extends StatelessWidget {
  const LogLevelSelector({super.key});

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    List<DropdownMenuItem> dropDownItems = [];
    for (var i = 0; i < logLevels.length; i++) {
      var value = logLevels[i];
      dropDownItems.add(DropdownMenuItem(
          value: value,
          child: Row(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [getLevelIcon(value), Text(value)],
          )));
    }
    return Row(
      children: [
        Padding(
          padding: const EdgeInsets.all(5.0),
          child: SizedBox(
              width: 30.0,
              height: 30.0,
              child: getLevelIcon(ourchatAppState.config["log_level"])),
        ),
        Expanded(
            child: DropdownButtonFormField(
                decoration: InputDecoration(label: Text(l10n.logLevel)),
                initialValue: ourchatAppState.config["log_level"],
                items: dropDownItems,
                selectedItemBuilder: (context) {
                  List<DropdownMenuItem> selectedItems = [];
                  for (var i = 0; i < logLevels.length; i++) {
                    var value = logLevels[i];
                    selectedItems.add(
                        DropdownMenuItem(value: value, child: Text(value)));
                  }
                  return selectedItems;
                },
                onChanged: (value) {
                  ourchatAppState.config["log_level"] = value;
                  ourchatAppState.update();
                })),
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
        return Icon(
          Icons.bug_report,
          size: size,
        );
      case 'info':
        return Icon(Icons.info, size: size);
      case 'warning':
        return Icon(
          Icons.warning,
          size: size,
        );
      case 'error':
        return Icon(
          Icons.error,
          size: size,
        );
      case 'fatal':
        return Icon(
          Icons.dangerous,
          size: size,
        );
      case 'trace':
        return Icon(
          Icons.code,
          size: size,
        );
      default:
        return Icon(Icons.help);
    }
  }
}
