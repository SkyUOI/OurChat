import 'dart:io';

/// IO variant of [platform.dart]: backed by dart:io's Platform.
bool get isDesktopOS =>
    Platform.isWindows || Platform.isLinux || Platform.isMacOS;

bool get isWindowsOS => Platform.isWindows;

/// Quit the whole application (desktop only).
void exitApp() => exit(0);
