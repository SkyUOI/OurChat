import 'package:flutter/material.dart';
import 'main.dart';
import 'package:provider/provider.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class Setting extends StatelessWidget {
  const Setting({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurchatAppState>();
    final formKey = GlobalKey<FormState>();
    return Scaffold(
        body: Center(
      child: Padding(
        padding: const EdgeInsets.only(top: 20.0, bottom: 20.0),
        child: Column(
          children: [
            Expanded(
              child: ListView(children: [
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
                                                  seedColor: Color(appState
                                                      .config!.data!["color"]))
                                              .secondary),
                                      color: Color(
                                          appState.config!.data!["color"])),
                                ),
                              ),
                            ),
                            Expanded(
                                child: TextFormField(
                                    decoration: InputDecoration(
                                        labelText: AppLocalizations.of(context)!
                                            .themeColorSeed),
                                    controller: TextEditingController(
                                        text:
                                            "0x${appState.config!.data!["color"].toRadixString(16)}"),
                                    validator: (value) {
                                      if (value == null || value.isEmpty) {
                                        return AppLocalizations.of(context)!
                                            .cantBeEmpty;
                                      }
                                      return null;
                                    },
                                    onSaved: (value) {
                                      appState.config!.data!["color"] =
                                          int.parse(value!);
                                    })),
                          ],
                        ),
                      ],
                    )),
              ]),
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Padding(
                  padding: const EdgeInsets.all(5.0),
                  child: ElevatedButton(
                      onPressed: () {
                        var servers = appState.config!.data!["servers"];
                        appState.config!.data =
                            appState.config!.getDefaultConfig();
                        appState.config!.data!["servers"] = servers;
                        appState.update();
                        appState.config!.saveConfig();
                      },
                      child: Text(AppLocalizations.of(context)!.reset)),
                ),
                Padding(
                  padding: const EdgeInsets.all(5.0),
                  child: ElevatedButton(
                    child: Text(AppLocalizations.of(context)!.save),
                    onPressed: () {
                      if (formKey.currentState!.validate()) {
                        formKey.currentState!.save();
                        appState.update();
                        appState.config!.saveConfig();
                      }
                    },
                  ),
                )
              ],
            )
          ],
        ),
      ),
    ));
  }
}
