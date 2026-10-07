/// Platform checks that are safe on the web.
///
/// [dart:io]'s [Platform] throws on any member access when compiled for web,
/// so it must stay behind a conditional import and never be touched on the
/// web build. Use [isDesktopOS]/[isWindowsOS] instead of `Platform.isX`, and
/// combine with `kIsWeb` (or [isDesktopPlatform] in main.dart) where the
/// desktop-only branch also needs to be excluded on web.
library;

export 'platform_web.dart' if (dart.library.io) 'platform_io.dart';
