import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/main.dart';

class Launch extends StatelessWidget {
  const Launch({
    super.key,
  });

  Future initConfig(OurChatAppState appState) async {
    appState.config.prefsWithCache = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions());
  }

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<OurChatAppState>();
    return MaterialApp(
      home: Scaffold(
        body: FutureBuilder(
            future: initConfig(appState),
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.done) {
                appState.config.reload();
                constructLogger(
                    convertStrIntoLevel(appState.config["log_level"]));
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  Navigator.pop(context);
                  Navigator.push(context,
                      MaterialPageRoute(builder: (context) => Controller()));
                });
              }
              return Center(
                child: CircularProgressIndicator(),
              );
            }),
      ),
    );
  }
}
