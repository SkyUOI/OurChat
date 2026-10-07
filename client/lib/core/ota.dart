/// In-app OTA update support, platform-gated (same conditional-import pattern
/// as core/platform.dart). The `ota_update` plugin has no web implementation
/// and imports dart:io, so the web build gets `ota_stub.dart` instead of
/// `ota_io.dart`. The update UI is unreachable on web anyway
/// (`enableVersionCheck` is false there) — this gate only keeps the imports
/// web-safe.
library;

export 'ota_stub.dart' if (dart.library.io) 'ota_io.dart';
