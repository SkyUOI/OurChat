import 'dart:io';

import 'package:device_info_plus/device_info_plus.dart';
import 'package:ota_update/ota_update.dart';

/// Release-asset name fragment for the current platform, matching the asset
/// names attached to a release (e.g. "windows.tar.gz", "android_arm64-v8a.apk").
Future<String> currentPlatformAssetName() async {
  if (Platform.isWindows) {
    return "windows.tar.gz";
  } else if (Platform.isLinux) {
    return "linux.tar.gz";
  } else if (Platform.isMacOS) {
    return "macos.tar.gz";
  } else if (Platform.isAndroid) {
    DeviceInfoPlugin deviceInfo = DeviceInfoPlugin();
    AndroidDeviceInfo androidInfo = await deviceInfo.androidInfo;
    String arch = androidInfo.supportedAbis.first;
    if (arch.contains('arm64')) {
      return "android_arm64-v8a.apk";
    } else if (arch.contains('armeabi')) {
      return "android_armeabi-v7a.apk";
    } else if (arch.contains('x86_64')) {
      return "android_x86_64.apk";
    } else {
      return "android_universal.apk";
    }
  } else if (Platform.isIOS) {
    return "ios";
  }
  return "";
}

/// Start downloading and installing the release package at [url], reporting
/// download progress as a 0..1 fraction (null while in a non-downloading
/// phase, e.g. installing).
Stream<double?> startOtaUpdate(String url) {
  OtaUpdate otaUpdate = OtaUpdate();
  Stream<OtaEvent> stream;
  if (Platform.isAndroid) {
    stream = otaUpdate.execute(
      url,
      destinationFilename: "OurChat.apk",
      usePackageInstaller: true,
    );
  } else {
    stream = otaUpdate.execute(url, destinationFilename: "OurChat.tar.gz");
  }
  return stream.map(
    (event) => event.status == OtaStatus.DOWNLOADING
        ? double.parse(event.value!)
        : null,
  );
}
