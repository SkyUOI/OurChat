import 'package:flutter/material.dart';
import 'package:ourchat/main.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'dart:io';
import 'package:flutter/foundation.dart';

class Launch extends StatefulWidget {
  const Launch({
    super.key,
  });

  @override
  State<Launch> createState() => _LaunchState();
}

class _LaunchState extends State<Launch> {
  Future initConfig(OurChatAppState appState) async {
    appState.config.prefsWithCache = await SharedPreferencesWithCache.create(
        cacheOptions: const SharedPreferencesWithCacheOptions());
  }

  bool inited = false;

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    return MaterialApp(
      scaffoldMessengerKey: rootScaffoldMessengerKey,
      localeResolutionCallback: (locale, supportedLocales) {
        Locale useLanguage = Locale("en");
        Locale? setLanguage = locale;
        if (ourchatAppState.config["language"] != null) {
          setLanguage = Locale.fromSubtags(
              languageCode: ourchatAppState.config["language"][0],
              scriptCode: ourchatAppState.config["language"][1],
              countryCode: ourchatAppState.config["language"][2]);
        }
        for (int i = 0; i < supportedLocales.length; i++) {
          var availableLanguage = supportedLocales.elementAt(i);
          if (availableLanguage.languageCode == setLanguage!.languageCode) {
            useLanguage = availableLanguage;
            break;
          }
        }
        logger.i(
            "use language (${useLanguage.languageCode},${useLanguage.scriptCode},${useLanguage.countryCode})");
        ourchatAppState.config["language"] = [
          useLanguage.languageCode,
          useLanguage.scriptCode,
          useLanguage.countryCode
        ];
        return useLanguage;
      },
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: FutureBuilder(
            future: initConfig(ourchatAppState),
            builder: (context, snapshot) {
              if (!inited && snapshot.connectionState == ConnectionState.done) {
                inited = true;
                ourchatAppState.config.reload();
                constructLogger(
                    convertStrIntoLevel(ourchatAppState.config["log_level"]));
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  Navigator.push(context,
                      MaterialPageRoute(builder: (context) => Controller()));
                });
              }
              return Center(
                child: CircularProgressIndicator(),
              );
            }),
      ),
      theme: ThemeData(
        fontFamily: kIsWeb ? null : (Platform.isWindows ? "微软雅黑" : null),
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Color(ourchatAppState.config["color"]),
        ),
      ),
    );
  }
}
