/// Web stub for the OTA updater (see ota.dart). Unreachable in practice: the
/// update UI is gated off on web via `enableVersionCheck` (core/const.dart).
Future<String> currentPlatformAssetName() async {
  throw UnsupportedError('OTA update is not supported on web');
}

Stream<double?> startOtaUpdate(String url) {
  throw UnsupportedError('OTA update is not supported on web');
}
