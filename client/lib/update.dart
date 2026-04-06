import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ota_update/ota_update.dart';
import "package:device_info_plus/device_info_plus.dart";
import 'package:ourchat/core/const.dart';
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
                    Stream<OtaEvent> stream;
                    if (Platform.isAndroid) {
                      OtaUpdate otaUpdate = OtaUpdate();
                      stream = otaUpdate.execute(
                        snapshot.data,
                        destinationFilename: "OurChat.apk",
                        usePackageInstaller: true,
                      );
                    } else {
                      OtaUpdate otaUpdate = OtaUpdate();
                      stream = otaUpdate.execute(
                        snapshot.data,
                        destinationFilename: "OurChat.tar.gz",
                      );
                    }
                    return StreamBuilder(
                      stream: stream,
                      builder: (context, snapshot) {
                        double? value;
                        if (snapshot.hasData) {
                          value =
                              (snapshot.data!.status == OtaStatus.DOWNLOADING
                              ? double.parse(snapshot.data!.value!)
                              : null);
                        }
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
    String platform = "";
    if (Platform.isWindows) {
      platform = "windows.tar.gz";
    } else if (Platform.isLinux) {
      platform = "linux.tar.gz";
    } else if (Platform.isMacOS) {
      platform = "macos.tar.gz";
    } else if (Platform.isAndroid) {
      DeviceInfoPlugin deviceInfo = DeviceInfoPlugin();
      AndroidDeviceInfo androidInfo = await deviceInfo.androidInfo;
      String arch = androidInfo.supportedAbis.first;
      if (arch.contains('arm64')) {
        platform = "android_arm64-v8a.apk";
      } else if (arch.contains('armeabi')) {
        platform = "android_armeabi-v7a.apk";
      } else if (arch.contains('x86_64')) {
        platform = "android_x86_64.apk";
      } else {
        platform = "android_universal.apk";
      }
    } else if (Platform.isIOS) {
      platform = "ios";
    }
    for (dynamic asset in widget.updateData["assets"]) {
      if (asset["name"].contains(platform)) {
        return asset["browser_download_url"];
      }
    }
    throw notFoundStatusCode;
  }
}
