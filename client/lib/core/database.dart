// Export the database classes from platform-specific implementations
export 'package:ourchat/core/database_web.dart'
    if (dart.library.io) 'package:ourchat/core/database_desktop.dart';
