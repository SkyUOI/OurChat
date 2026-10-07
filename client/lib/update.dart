import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/ota.dart';
import 'main.dart';

class UpdateWidget extends ConsumerStatefulWidget {
  final dynamic updateData;
  const UpdateWidget({super.key, required this.updateData});

  @override
  ConsumerState<UpdateWidget> createState() => _UpdateWidgetState();
}

class _UpdateWidgetState extends ConsumerState<UpdateWidget> {
  @override
  Widget build(BuildContext context) {
    String? text;
    return SafeArea(
      child: Scaffold(
        body: Column(
          children: [
            Row(children: [BackButton()]),
            Expanded(
              child: FutureBuilder(
                future: getDownloadInfo(),
                builder: (context, snapshot) {
                  if (snapshot.connectionState == ConnectionState.done &&
                      snapshot.data != null) {
                    return StreamBuilder<double?>(
                      stream: startOtaUpdate(snapshot.data as String),
                      builder: (context, snapshot) {
                        double? value = snapshot.data;
                        return Center(
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              CircularProgressIndicator(value: value),
                              Text(l10n.updateDownloading),
                            ],
                          ),
                        );
                      },
                    );
                  } else if (snapshot.connectionState != ConnectionState.done ||
                      snapshot.data == null) {
                    text = l10n.updateGettingInfo;
                  } else if (snapshot.hasError) {
                    if (snapshot.error == notFoundStatusCode) {
                      text = l10n.notFound(l10n.installationPackage);
                    }
                  }

                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        CircularProgressIndicator(value: 0),
                        Text(text!),
                      ],
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future getDownloadInfo() async {
    String platform = await currentPlatformAssetName();
    for (dynamic asset in widget.updateData["assets"]) {
      if (asset["name"].contains(platform)) {
        return asset["browser_download_url"];
      }
    }
    throw notFoundStatusCode;
  }
}
