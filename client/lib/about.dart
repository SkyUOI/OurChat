import 'package:flutter/material.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/chore.dart';
import 'package:flutter/services.dart';

// 将会在生成发行版时由脚本填入贡献者&赞助者名单
// ===== AUTO GENERATED CODE BEGIN =====
const List<Map<String, String>> contributorsList = [
  {
    "user": "limuy2022",
    "avatar": "https://avatars.githubusercontent.com/u/97649454?v=4",
    "url": "https://github.com/limuy2022"
  },
  {
    "user": "senlinjun",
    "avatar": "https://avatars.githubusercontent.com/u/78007298?v=4",
    "url": "https://github.com/senlinjun"
  },
  {
    "user": "OMObuan",
    "avatar": "https://avatars.githubusercontent.com/u/150120115?v=4",
    "url": "https://github.com/OMObuan"
  },
  {
    "user": "liya123",
    "avatar": "https://avatars.githubusercontent.com/u/12387755?v=4",
    "url": "https://github.com/liya123"
  }
];
const List<Map<String, String>> donorsList = [];
const version = "v0.1.0.beta";
const commitSha = "4c31dd44a1cbe7e454f26b359408b0e4201f8780";
// ===== AUTO GENERATED CODE END =====

class About extends StatefulWidget {
  const About({
    super.key,
  });

  @override
  State<About> createState() => _AboutState();
}

class _AboutState extends State<About> {
  bool showFullCommit = false;
  @override
  Widget build(BuildContext context) {
    var l10n = AppLocalizations.of(context)!;
    var ourchatAppState = context.watch<OurChatAppState>();
    Widget info = Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("Version: "),
            Text(version, style: TextStyle(fontSize: 20)),
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("Commit: "),
            InkWell(
              onTap: () {
                Clipboard.setData(ClipboardData(text: commitSha));
                ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.copiedToClipboard)));
              },
              child: Text(commitSha.substring(0, 7),
                  style: TextStyle(fontSize: 20)),
            ),
          ],
        ),
      ],
    );
    Widget contributors = ListView.builder(
      itemBuilder: (context, index) {
        return Column(
          children: [
            InkWell(
              child: Column(
                children: [
                  Row(
                    children: [
                      CircleAvatar(
                          radius: 20.0,
                          backgroundImage:
                              NetworkImage(contributorsList[index]["avatar"]!)),
                      Text(contributorsList[index]["user"]!),
                    ],
                  ),
                ],
              ),
              onTap: () {
                launchUrl(Uri.parse(contributorsList[index]["url"]!));
              },
            ),
            Divider()
          ],
        );
      },
      itemCount: contributorsList.length,
    );
    Widget donors = ListView.builder(
      itemBuilder: (context, index) {
        return Column(
          children: [
            InkWell(
                child: Column(
              children: [
                Row(
                  children: [
                    CircleAvatar(
                        radius: 20.0,
                        backgroundImage:
                            NetworkImage(donorsList[index]["avatar"]!)),
                    Text(donorsList[index]["user"]!),
                  ],
                ),
              ],
            )),
            Divider()
          ],
        );
      },
      itemCount: donorsList.length,
    );
    if (ourchatAppState.device == mobile) {
      return DefaultTabController(
          length: 3,
          child: Scaffold(
            appBar: TabBar(tabs: [
              Tab(
                text: l10n.about,
              ),
              Tab(text: l10n.donate),
              Tab(text: l10n.contribute)
            ]),
            body: Column(
              children: [
                Expanded(
                  child: TabBarView(children: [
                    Column(
                      mainAxisAlignment: MainAxisAlignment.spaceAround,
                      children: [
                        SizedBox(
                            width: 200,
                            child: Image.asset(
                              "assets/images/logo.png",
                            )),
                        info
                      ],
                    ),
                    Padding(
                      padding: const EdgeInsets.all(10.0),
                      child: Card(
                        child: Stack(
                          children: [
                            donors,
                            Align(
                              alignment: Alignment.bottomRight,
                              child: FloatingActionButton(
                                onPressed: () {
                                  launchUrl(Uri.parse(
                                      "https://www.afdian.com/a/ourchat"));
                                },
                                child: Icon(Icons.coffee),
                              ),
                            )
                          ],
                        ),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(5.0),
                      child: Card(
                        child: Stack(
                          children: [
                            contributors,
                            Align(
                              alignment: Alignment.bottomRight,
                              child: FloatingActionButton(
                                onPressed: () {
                                  launchUrl(Uri.parse(
                                      "https://github.com/skyuoi/ourchat"));
                                },
                                child: Icon(Icons.code),
                              ),
                            )
                          ],
                        ),
                      ),
                    )
                  ]),
                ),
                Row(
                  children: [
                    BackButton(onPressed: () => Navigator.pop(context)),
                  ],
                )
              ],
            ),
          ));
    }

    // Desktop
    return Scaffold(
      body: Column(
        children: [
          Row(
            children: [
              BackButton(onPressed: () => Navigator.pop(context)),
            ],
          ),
          Expanded(
            child: SingleChildScrollView(
              child: Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      SizedBox(
                          width: 600,
                          child: Image.asset(
                            "assets/images/logo.png",
                          )),
                      Expanded(
                        child: info,
                      )
                    ],
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      Column(
                        children: [
                          ElevatedButton.icon(
                              style: AppStyles.defaultButtonStyle,
                              onPressed: () {
                                launchUrl(Uri.parse(
                                    "https://github.com/skyuoi/ourchat"));
                              },
                              label: Text("Github"),
                              icon: Icon(Icons.code)),
                          Card(
                            child: Column(
                              children: [
                                Padding(
                                  padding: const EdgeInsets.only(bottom: 10.0),
                                  child: Text(
                                    "Contributors",
                                    style: TextStyle(fontSize: 25),
                                  ),
                                ),
                                SizedBox(
                                    height: 500,
                                    width: 400,
                                    child: contributors)
                              ],
                            ),
                          )
                        ],
                      ),
                      Column(
                        children: [
                          ElevatedButton.icon(
                              style: AppStyles.defaultButtonStyle,
                              onPressed: () {
                                launchUrl(Uri.parse(
                                    "https://www.afdian.com/a/ourchat"));
                              },
                              label: Text(l10n.donate),
                              icon: Icon(Icons.coffee)),
                          Card(
                            child: Column(
                              children: [
                                Padding(
                                  padding: const EdgeInsets.only(bottom: 10.0),
                                  child: Text(
                                    "Donor",
                                    style: TextStyle(fontSize: 25),
                                  ),
                                ),
                                SizedBox(height: 500, width: 400, child: donors)
                              ],
                            ),
                          )
                        ],
                      )
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
