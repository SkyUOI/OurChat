import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:ourchat/core/const.dart';

List analyzeVersionString(String version) {
  List<String> versionList = version.replaceAll("v", "").split(".");
  int latestX, latestY, latestZ;
  latestX = int.parse(versionList[0]);
  latestY = int.parse(versionList[1]);
  latestZ = int.parse(versionList[2].replaceAll(RegExp("-.*"), ""));
  String other = version.replaceAll("v$latestX.$latestY.$latestZ-", "");
  return [
    latestX,
    latestY,
    latestZ,
    other,
    other.contains("alpha"),
    other.contains("beta"),
  ];
}

Future needUpdate(Uri source, bool acceptAlpha, bool acceptBeta) async {
  http.Response res = await http.get(source);
  var data = jsonDecode(res.body);
  for (int i = 0; i < data.length; i++) {
    String? version = data[i]["tag_name"];
    if (version == null) return null;
    if (version == currentVersion) return null;
    List latestVersionList = analyzeVersionString(version);
    List currentVersionList = analyzeVersionString(currentVersion);
    for (int j = 0; j < 3; j++) {
      if (latestVersionList[j] > currentVersionList[j] &&
          (acceptAlpha || !latestVersionList[4]) &&
          (acceptBeta || !latestVersionList[5])) {
        return data[i];
      } else if (latestVersionList[j] < currentVersionList[j]) {
        return null;
      }
    }
    if (latestVersionList[4] && acceptAlpha) {
      return data[i];
    }
    if (latestVersionList[5] && acceptBeta) {
      return data[i];
    }
  }
  return null;
}
