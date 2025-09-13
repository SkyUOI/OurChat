import 'package:flutter/material.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/about.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'main.dart';

class User extends StatelessWidget {
  const User({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    return Expanded(
      child: Column(
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
              Text("${l10n.email}: "),
              SelectableText(appState.thisAccount!.email!),
            ],
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text("${l10n.ocid}: "),
              SelectableText(appState.thisAccount!.ocid),
            ],
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                  onPressed: () {
                    appState.thisAccount = null;
                    appState.server = null;
                    appState.privateDB!.close();
                    appState.privateDB = null;
                    appState.accountCachePool = {};
                    appState.sessionCachePool = {};
                    appState.eventSystem!.stopListening();
                    Navigator.pop(context);
                    Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => ServerSetting()));
                  },
                  child: Text(l10n.logout)),
              ElevatedButton(
                  onPressed: () {
                    Navigator.push(context,
                        MaterialPageRoute(builder: (context) => About()));
                  },
                  child: Text(l10n.about))
            ],
          ),
        ],
      ),
    );
  }
}
