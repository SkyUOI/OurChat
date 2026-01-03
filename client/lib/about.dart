import 'package:flutter/material.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/update.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/chore.dart';
import 'package:flutter/services.dart';

class About extends StatefulWidget {
  const About({
    super.key,
  });

  @override
  State<About> createState() => _AboutState();
}

class _AboutState extends State<About> {
  bool showFullCommit = false, isNeedUpdate = false, inited = false;
  dynamic updateData;
  @override
  Widget build(BuildContext context) {
    var l10n = AppLocalizations.of(context)!;
    var ourchatAppState = context.watch<OurChatAppState>();
    if (!inited) {
      if (enableVersionCheck) {
        needUpdate(
                Uri.parse(ourchatAppState.config["update_source"]), true, true)
            .then((value) {
          setState(() {
            updateData = value;
            isNeedUpdate = value != null;
          });
        });
      }
      inited = true;
    }
    Widget info = Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.start,
          children: [
            Text("Version: "),
            Text(currentVersion, style: TextStyle(fontSize: 20)),
            if (enableVersionCheck && isNeedUpdate)
              Padding(
                padding: const EdgeInsets.all(AppStyles.smallPadding),
                child: ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  onPressed: () {
                    Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => UpdateWidget(
                                  updateData: updateData,
                                )));
                  },
                  icon: Icon(Icons.system_update),
                  label: Text(
                    l10n.newVersionAvailable,
                    style: TextStyle(fontSize: 13),
                  ),
                ),
              ),
            if (!enableVersionCheck)
              Padding(
                  padding: const EdgeInsets.all(AppStyles.smallPadding),
                  child: Text(
                    l10n.disableUpdate,
                    style: TextStyle(
                        fontSize: 13, color: Theme.of(context).hintColor),
                  ))
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.start,
          children: [
            Text("Commit: "),
            InkWell(
              onTap: () {
                Clipboard.setData(ClipboardData(text: currentCommitSha));
                ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.copiedToClipboard)));
              },
              child: Text(currentCommitSha.substring(0, 7),
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
    if (ourchatAppState.screenMode == mobile) {
      return SafeArea(
        child: DefaultTabController(
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
                      BackButton(),
                    ],
                  )
                ],
              ),
            )),
      );
    }

    // Desktop
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Row(
              children: [
                BackButton(),
              ],
            ),
            Expanded(
              child: SingleChildScrollView(
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                      children: [
                        SizedBox(
                            width: 600,
                            child: Image.asset(
                              "assets/images/logo.png",
                            )),
                        Expanded(
                          child: Center(child: info),
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
                                    padding:
                                        const EdgeInsets.only(bottom: 10.0),
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
                                    padding:
                                        const EdgeInsets.only(bottom: 10.0),
                                    child: Text(
                                      "Donor",
                                      style: TextStyle(fontSize: 25),
                                    ),
                                  ),
                                  SizedBox(
                                      height: 500, width: 400, child: donors)
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
      ),
    );
  }
}
