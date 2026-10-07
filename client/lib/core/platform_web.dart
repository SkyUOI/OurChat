/// Web variant of [platform.dart]: answered statically so the web build never
/// touches dart:io (whose Platform throws on any access there).
bool get isDesktopOS => false;

bool get isWindowsOS => false;

/// No process to exit on web; closing the tab is the user's own action.
void exitApp() {}
