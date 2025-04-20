import 'package:flutter/material.dart';
import 'package:ourchat/config.dart';
import 'package:ourchat/log.dart';
import 'main.dart';
import 'package:provider/provider.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class Setting extends StatelessWidget {
  const Setting({super.key});

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurchatAppState>();
    final formKey = GlobalKey<FormState>();
    var i18n = AppLocalizations.of(context)!;
    var originalColor = ourchatConfig["color"];

    return Scaffold(
      body: Center(
        child: Padding(
          padding: const EdgeInsets.only(top: 20.0, bottom: 20.0),
          child: Column(
            children: [
              Expanded(
                child: ListView(
                  children: [
                    Form(
                      key: formKey,
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Row(
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
                                          seedColor: Color(originalColor),
                                        ).secondary,
                                      ),
                                      color: Color(originalColor),
                                    ),
                                  ),
                                ),
                              ),
                              Expanded(
                                child: TextFormField(
                                  decoration: InputDecoration(
                                    labelText: i18n.themeColorSeed,
                                  ),
                                  controller: TextEditingController(
                                    text:
                                        "0x${ourchatConfig["color"].toRadixString(16)}",
                                  ),
                                  validator: (value) {
                                    if (value == null || value.isEmpty) {
                                      return i18n.cantBeEmpty;
                                    }
                                    return null;
                                  },
                                  onSaved: (value) {
                                    ourchatConfig["color"] = int.parse(
                                      value!,
                                    );
                                  },
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                    _LogLevelSelector()
                  ],
                ),
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Padding(
                    padding: const EdgeInsets.all(5.0),
                    child: ElevatedButton(
                      onPressed: () {
                        var servers = ourchatConfig["servers"];
                        ourchatConfig
                            .set(ourchatConfig.data.getDefaultConfig());
                        ourchatConfig["servers"] = servers;
                        appState.update();
                      },
                      child: Text(i18n.reset),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.all(5.0),
                    child: ElevatedButton(
                      child: Text(i18n.save),
                      onPressed: () {
                        if (formKey.currentState!.validate()) {
                          formKey.currentState!.save();
                          appState.update();
                        }
                      },
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _LogLevelSelector extends StatefulWidget {
  @override
  _LogLevelSelectorState createState() => _LogLevelSelectorState();
}

class _LogLevelSelectorState extends State<_LogLevelSelector> {
  String _selectedLevel = ourchatConfig["log_level"];

  @override
  Widget build(BuildContext context) {
    var i18n = AppLocalizations.of(context)!;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          _getLevelIcon(_selectedLevel),
          Expanded(
            child: InputDecorator(
              decoration: InputDecoration(
                labelText: i18n.logLevel,
                isCollapsed: true,
                contentPadding: EdgeInsets.only(bottom: 4),
                border: UnderlineInputBorder(
                  borderSide: BorderSide(
                    color: Theme.of(context).dividerColor,
                    width: 1.0,
                  ),
                  borderRadius: BorderRadius.zero,
                ),
                enabledBorder: UnderlineInputBorder(
                  borderSide: BorderSide(
                    color: Theme.of(context).dividerColor,
                    width: 1.0,
                  ),
                  borderRadius: BorderRadius.zero,
                ),
                alignLabelWithHint: true,
                isDense: true,
              ),
              child: Padding(
                padding: const EdgeInsets.only(top: 8.0),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<String>(
                    isExpanded: true,
                    value: _selectedLevel,
                    icon: const Icon(Icons.arrow_drop_down, size: 24),
                    style: Theme.of(context).textTheme.bodyLarge,
                    onChanged: (String? newValue) {
                      setState(() {
                        _selectedLevel = newValue!;
                      });
                      ourchatConfig["log_level"] = _selectedLevel;
                      constructLogger();
                      logger.i('Selected log level: $_selectedLevel');
                    },
                    items:
                        logLevels.map<DropdownMenuItem<String>>((String value) {
                      return DropdownMenuItem<String>(
                        value: value,
                        child: Row(
                          children: [
                            SizedBox(width: 8),
                            Text(value),
                          ],
                        ),
                      );
                    }).toList(),
                  ),
                ),
              ),
            ),
          )
        ],
      ),
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
  Icon _getLevelIcon(String level) {
    var size = 44.0;
    switch (level) {
      case 'debug':
        return Icon(
          Icons.bug_report,
          color: Colors.green,
          size: size,
        );
      case 'info':
        return Icon(Icons.info, color: Colors.blue, size: size);
      case 'warning':
        return Icon(
          Icons.warning,
          color: Colors.orange,
          size: size,
        );
      case 'error':
        return Icon(
          Icons.error,
          color: Colors.red,
          size: size,
        );
      case 'fatal':
        return Icon(
          Icons.dangerous,
          color: Colors.purple,
          size: size,
        );
      case 'trace':
        return Icon(
          Icons.code,
          color: Colors.grey,
          size: size,
        );
      default:
        return Icon(Icons.help);
    }
  }
}
