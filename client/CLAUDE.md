# CLAUDE.md - Client

This file provides guidance to Claude Code (claude.ai/code) when working with the **client** portion of this repository.

## Project Overview

OurChat client is a cross-platform chat application built with Flutter, supporting Linux, Windows, macOS, Android, iOS, and web. It connects to the OurChat server via gRPC (with TLS support) and provides real-time messaging, group chats, end-to-end encryption, and a modern user interface.

## Quick Reference

### Most Common Commands

```bash
# Run on connected device/emulator
flutter run

# Build for specific platform
flutter build linux   # or windows, macos, apk, ios, web

# Generate protobuf/gRPC code
python script/generate.pb.dart.py

# Generate database code
dart run build_runner build --delete-conflicting-outputs
```

### Critical Notes

- **Cross-platform**: Test on all target platforms when making changes
- **State management**: Centralized in `OurChatAppState` (Provider)
- **Database**: Uses `drift` with SQLite, separate public/private databases
- **gRPC**: Generated code in `service/` directory from `.proto` files
- **Internationalization**: All user-facing strings via ARB files
- **Code generation**: Do not edit generated `*.pb.dart` or `*.g.dart` files
- See sections below for detailed guidance

## Development Environment

### Flutter & Dart

- **Flutter SDK**: Version compatible with Dart 3.5.3 (see `pubspec.yaml`)
- **Dart**: SDK ^3.5.3
- **Platforms**: Android, iOS, Linux, Windows, macOS, web

### Dependencies

Key dependencies (see `pubspec.yaml` for full list):

- **provider**: State management
- **grpc** & **protobuf**: Server communication
- **drift** & **drift_flutter**: Local SQLite database
- **shared_preferences**: Persistent settings
- **window_manager** & **tray_manager**: Desktop window/tray integration
- **image_picker**: Image selection
- **cached_network_image**: Efficient image caching
- **intl** & **flutter_localizations**: Internationalization

### Build Commands

**Code Generation:**

```bash
# Generate protobuf/gRPC Dart code from .proto files
python script/generate.pb.dart.py

# Generate drift database code (if database schema changed)
dart run build_runner build --delete-conflicting-outputs

# Format code
dart format lib/
```

## Platform-Specific Builds

### Linux

- Requires GTK3 development libraries
- AppImages can be generated with `flutter build linux`
- System tray integration via `tray_manager`

### Windows

- Requires Visual Studio build tools
- MSIX/AppX packaging available
- System tray integration with `.ico` icons

### macOS

- Requires Xcode and CocoaPods
- Notarization required for distribution
- Menu bar integration supported

### Android

- Requires Android SDK and NDK
- Keystore required for release builds
- Firebase integration possible

### iOS

- Requires Xcode and macOS
- App Store distribution requires Apple Developer account
- Push notification configuration needed

### Web

- Requires modern browser with WebAssembly support
- gRPC-Web via Envoy proxy
- Service Worker for offline functionality

## Architecture Overview

### Directory Structure

- `lib/` - Main application source
  - `core/` - Core functionality (config, database, server, session, account, event, log, UI constants)
  - `service/` - Generated gRPC service code (from protobuf definitions)
  - Top-level Dart files: `main.dart`, `auth.dart`, `home.dart`, `friends.dart`, `setting.dart`, `about.dart`, `server_setting.dart`, `session.dart`, `launch.dart`

### Core Components

1. **State Management**: Uses `Provider` with `OurChatAppState` as the central app state.
2. **Local Database**: `drift` for SQLite, with separate public and private databases.
3. **Server Communication**: `OurChatServer` class handles gRPC connections, TLS detection, and service calls.
4. **Authentication**: `OurChatAccount` manages login, token refresh, and account info.
5. **Session Management**: `OurChatSession` represents chat sessions.
6. **Event System**: `OurChatEventSystem` listens to server events (new messages, etc.).
7. **Configuration**: `OurChatConfig` loads/saves user settings via `shared_preferences`.
8. **Internationalization**: `AppLocalizations` for multi-language support.
9. **Desktop Integration**: `window_manager` and `tray_manager` for desktop-specific features.

### UI Structure

- `Launch` widget handles initial routing (auto-login, server setting, auth, or home).
- `Home` is the main chat interface.
- `Auth` handles login/registration.
- `ServerSetting` configures server connection.
- `Friends`, `Setting`, `About`, `Session` provide additional functionality.

## Key Configuration

### Configuration Files

- **User settings**: Stored via `shared_preferences` (platform-specific location).
- **Server configuration**: Host, port, TLS settings in config.
- **Assets**: Images in `assets/images/`, localized strings via ARB files.

### Environment Variables

- No environment variables required for client; all configuration is user-managed.

## Development Workflow

### Testing

- Unit tests can be added in `test/` directory.
- Integration tests in `integration_test/`.
- Run with `flutter test`.

### Code Generation

When protobuf definitions change (`service/` directory):

1. Run `python script/generate.pb.dart.py` to regenerate Dart gRPC code.
2. Ensure `protoc` with Dart plugin is installed.

When database schema changes (`lib/core/database.dart`):

1. Update the drift table definitions.
2. Run `dart run build_runner build --delete-conflicting-outputs`.
3. Commit the generated `.g.dart` files.

### Code Quality

- Uses `flutter_lints` for static analysis.
- Format code with `dart format`.
- Follow Flutter best practices and the existing project patterns.

## Common Development Tasks

### Adding a New UI Screen

1. Create a new Dart file in `lib/` (e.g., `new_screen.dart`).
2. Define a `StatefulWidget`/`StatelessWidget`.
3. Add navigation to/from the screen in appropriate existing widgets.
4. Update `OurChatAppState` if new state is needed.
5. Add translations in ARB files if text needs localization.

### Integrating a New gRPC Service

1. Ensure the `.proto` file is in `service/` directory (shared with server).
2. Regenerate Dart code with `python script/generate.pb.dart.py`.
3. Add methods to `OurChatServer` class to call the new service.
4. Update UI to use the new functionality.

### Updating Local Database Schema

1. Modify table definitions in `lib/core/database.dart`.
2. Run `dart run build_runner build --delete-conflicting-outputs`.
3. Update any queries that depend on the changed schema.
4. Handle migrations if needed (drift provides migration support).

### Adding Internationalization

1. Add new keys to `lib/l10n/app_en.arb` (and other language files).
2. Run `flutter gen-l10n` to regenerate localization classes.
3. Use `AppLocalizations.of(context)!.keyName` in widgets.

### Desktop-Specific Features

- Use `Platform.isWindows`, `Platform.isLinux`, `Platform.isMacOS` to conditionally include desktop code.
- `window_manager` for window control (minimize, close, etc.).
- `tray_manager` for system tray icons and menus.

## Troubleshooting

### Connection Issues

- **Server unreachable**: Check server address/port in settings, verify server is running
- **TLS errors**: Ensure server TLS certificate is valid, or disable TLS for development
- **gRPC errors**: Verify protobuf definitions match between client and server

### Build Issues

- **Flutter version mismatch**: Check `pubspec.yaml` for Dart SDK constraints
- **Missing native dependencies**: Install platform-specific build tools (see Platform-Specific Builds)
- **Code generation failures**: Run `python script/generate.pb.dart.py` and `dart run build_runner build`

### Runtime Issues

- **Database errors**: Delete `~/.local/share/ourchat/` (Linux) or app data directory to reset local database
- **UI rendering problems**: Check Flutter channel compatibility, try `flutter clean`
- **Internationalization missing**: Run `flutter gen-l10n` to regenerate localization files

### Platform-Specific Issues

- **Linux tray not appearing**: Install `libappindicator` or `libayatana-appindicator`
- **Windows window management**: Ensure `window_manager` permissions in manifest
- **macOS permissions**: Grant necessary entitlements for network, tray, etc.

## Important Notes

- The client is **cross-platform**; test on all target platforms when making changes.
- **State management** is centralized in `OurChatAppState`; avoid creating additional providers unless necessary.
- **Database operations** are async; use `await` and handle errors appropriately.
- **gRPC calls** may fail due to network issues; implement retry/timeout logic.
- **UI responsiveness**: The app adapts to mobile/desktop via `screenMode` based on aspect ratio.
- **Internationalization**: All user-facing strings should be localized via ARB files.
- **Assets**: Image assets are in `assets/images/`; update `pubspec.yaml` if adding new asset directories.
- **Code generation**: Generated files (`*.pb.dart`, `*.g.dart`) should not be edited manually.
