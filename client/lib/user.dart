import 'package:flutter/material.dart';
import 'package:ourchat/server_setting.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'main.dart';

class User extends StatelessWidget {
  const User({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurchatAppState>();
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.all(10.0),
          child: SizedBox(
            width: 100.0,
            height: 100.0,
            child: Placeholder(),
          ),
        ),
        Text(
          appState.thisAccount!.username,
          style: TextStyle(fontSize: 20),
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${AppLocalizations.of(context)!.email}: "),
            SelectableText(appState.thisAccount!.email),
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${AppLocalizations.of(context)!.ocid}: "),
            SelectableText(appState.thisAccount!.ocid),
          ],
        ),
        ElevatedButton(
            onPressed: () {
              appState.thisAccount = null;
              appState.server = null;
              appState.privateDB = null;
              appState.privateDB!.close();
              Navigator.pop(context);
              Navigator.push(context, MaterialPageRoute(builder: (context) {
                return ServerSetting();
              }));
            },
            child: Text(AppLocalizations.of(context)!.logout))
      ],
    );
  }
}
