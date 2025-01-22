import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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
                        TextFormField(
                          decoration: InputDecoration(
                              labelText:
                                  AppLocalizations.of(context)!.serverAddress),
                          controller: TextEditingController(
                              text: appState.config!.data!["server"][0]
                                  ["host"]),
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return AppLocalizations.of(context)!.cantBeEmpty;
                            }
                            return null;
                          },
                          onSaved: (value) {
                            appState.config!.data!["server"][0]["host"] = value;
                          },
                        ),
                        TextFormField(
                          decoration: InputDecoration(
                              labelText: AppLocalizations.of(context)!.port),
                          controller: TextEditingController(
                              text: appState.config!.data!["server"][0]["port"]
                                  .toString()),
                          inputFormatters: [
                            FilteringTextInputFormatter.allow(RegExp(r'[0-9]')),
                          ],
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return AppLocalizations.of(context)!.cantBeEmpty;
                            }
                            if (int.parse(value) < 0 ||
                                int.parse(value) > 65535) {
                              return AppLocalizations.of(context)!
                                  .notWithinRange(0, 65535);
                            }
                            return null;
                          },
                          onSaved: (value) {
                            appState.config!.data!["server"][0]["port"] = value;
                          },
                        ),
                        TextFormField(
                          decoration: InputDecoration(
                              labelText: AppLocalizations.of(context)!
                                  .reconnectionAttempt),
                          controller: TextEditingController(
                              text: appState
                                  .config!.data!["reconnection_attempt"]
                                  .toString()),
                          inputFormatters: [
                            FilteringTextInputFormatter.allow(RegExp(r'[0-9]')),
                          ],
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return AppLocalizations.of(context)!.cantBeEmpty;
                            }
                            return null;
                          },
                          onSaved: (value) {
                            appState.config!.data!["reconnection_attempt"] =
                                value;
                          },
                        ),
                        TextFormField(
                          decoration: InputDecoration(
                              labelText: AppLocalizations.of(context)!
                                  .reconnectionInterval),
                          controller: TextEditingController(
                              text: appState
                                  .config!.data!["reconnection_interval"]
                                  .toString()),
                          inputFormatters: [
                            FilteringTextInputFormatter.allow(RegExp(r'[0-9]')),
                          ],
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return AppLocalizations.of(context)!.cantBeEmpty;
                            }
                            return null;
                          },
                          onSaved: (value) {
                            appState.config!.data!["reconnection_interval"] =
                                value;
                          },
                        ),
                      ],
                    )),
              ]),
            ),
            Align(
              alignment: Alignment.bottomCenter,
              child: ElevatedButton(
                child: const Text("Save"),
                onPressed: () {
                  if (formKey.currentState!.validate()) {
                    formKey.currentState!.save();
                    appState.config!.saveConfig();
                  }
                },
              ),
            )
          ],
        ),
      ),
    ));
  }
}
