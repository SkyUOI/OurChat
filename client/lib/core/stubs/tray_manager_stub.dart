// Stub implementation of tray_manager for web platform
class TrayManager {
  static final TrayManager _instance = TrayManager._internal();
  factory TrayManager() => _instance;
  TrayManager._internal();

  Future<void> ensureInitialized() async {
    // No-op for web
  }

  Future<void> addListener(TrayListener listener) async {
    // No-op for web
  }

  Future<void> removeListener(TrayListener listener) async {
    // No-op for web
  }

  Future<void> setIcon(String iconPath) async {
    // No-op for web
  }

  Future<void> setContextMenu(Menu menu) async {
    // No-op for web
  }

  Future<void> popUpContextMenu() async {
    // No-op for web
  }

  Future<void> destroy() async {
    // No-op for web
  }
}

class Menu {
  final List<MenuItem> items;
  const Menu({required this.items});
}

class MenuItem {
  final String key;
  final String label;
  const MenuItem({required this.key, required this.label});
}

mixin TrayListener {
  void onTrayIconRightMouseDown() {}
  void onTrayMenuItemClick(MenuItem menuItem) {}
  void onTrayIconMouseDown() {}
}

final trayManager = TrayManager();
