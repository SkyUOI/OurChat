// Stub implementation of window_manager for web platform
class WindowManager {
  static final WindowManager _instance = WindowManager._internal();
  factory WindowManager() => _instance;
  WindowManager._internal();

  Future<void> ensureInitialized() async {
    // No-op for web
  }

  Future<void> addListener(WindowListener listener) async {
    // No-op for web
  }

  Future<void> removeListener(WindowListener listener) async {
    // No-op for web
  }

  Future<void> setPreventClose(bool preventClose) async {
    // No-op for web
  }

  Future<void> waitUntilReadyToShow(
      WindowOptions options, Function() callback) async {
    // No-op for web
  }

  Future<void> show() async {
    // No-op for web
  }

  Future<void> hide() async {
    // No-op for web
  }

  Future<void> focus() async {
    // No-op for web
  }

  Future<void> destroy() async {
    // No-op for web
  }
}

class WindowOptions {
  const WindowOptions({
    this.minimumSize,
    this.center = false,
    this.skipTaskbar = false,
    this.title = "",
  });

  final Size? minimumSize;
  final bool center;
  final bool skipTaskbar;
  final String title;
}

class Size {
  const Size(this.width, this.height);
  final double width;
  final double height;
}

mixin WindowListener {
  void onWindowClose() {}
}

final windowManager = WindowManager();
