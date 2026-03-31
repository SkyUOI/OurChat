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

# Generate database code + Riverpod/freezed/json_serializable code
dart run build_runner build --delete-conflicting-outputs
```

### Critical Notes

- **Cross-platform**: Test on all target platforms when making changes
- **State management**: Riverpod with code generation (`riverpod_annotation` + `riverpod_generator`)
- **Data classes**: `freezed` for immutable models with code generation
- **Database**: Uses `drift` with SQLite, separate public/private databases
- **gRPC**: Generated code in `lib/service/` directory from `.proto` files
- **Internationalization**: All user-facing strings via ARB files
- **Code generation**: Do not edit generated `*.pb.dart`, `*.g.dart`, or `*.freezed.dart` files
- See sections below for detailed guidance

## Development Environment

### Flutter & Dart

- **Dart SDK**: ^3.8.0
- **Platforms**: Android, iOS, Linux, Windows, macOS, web

### Dependencies

Key dependencies (see `pubspec.yaml` for full list):

- **flutter_riverpod** + **hooks_riverpod** + **riverpod_annotation**: State management with code generation
- **grpc** & **protobuf**: Server communication
- **drift** & **drift_flutter**: Local SQLite database
- **freezed_annotation** + **freezed**: Immutable data classes with union types
- **json_annotation** + **json_serializable**: JSON serialization
- **shared_preferences**: Persistent settings
- **window_manager** & **tray_manager**: Desktop window/tray integration
- **image_picker**: Image selection
- **cached_network_image**: Efficient image caching
- **flutter_markdown_plus**: Markdown rendering
- **intl** & **flutter_localizations**: Internationalization
- **hashlib**: SHA-3 hashing for file uploads

### Code Generation Tools (dev_dependencies)

- **riverpod_generator**: Generates Riverpod providers from annotations
- **freezed**: Generates immutable data classes
- **json_serializable**: Generates JSON serialization code
- **drift_dev**: Generates database code
- **build_runner**: Orchestrates all code generation

### Build Commands

**Code Generation:**

```bash
# Generate protobuf/gRPC Dart code from .proto files
python script/generate.pb.dart.py

# Generate all code (drift, riverpod, freezed, json_serializable)
dart run build_runner build --delete-conflicting-outputs

# Format code
dart format lib/
```

## Architecture Overview

### Directory Structure

```
lib/
├── main.dart              # App entry point, Riverpod providers for core state
├── home.dart              # Main navigation shell (desktop: NavigationRail, mobile: BottomNavigationBar)
├── auth.dart              # Login/registration screen
├── session.dart           # Chat session list and messaging UI
├── friends.dart           # Friend management screen
├── setting.dart           # App settings screen
├── server_setting.dart    # Server connection configuration
├── user.dart              # User profile ("Me" tab)
├── about.dart             # About page (contributors, version info)
├── update.dart            # Update checking logic
├── core/
│   ├── config.dart        # OurChatConfig (freezed), ServerConfig, ConfigNotifier (Riverpod)
│   ├── auth_notifier.dart # AuthState (freezed), AuthNotifier (Riverpod) - login/register/logout
│   ├── account.dart       # AccountData (freezed), OurChatAccount provider - user data management
│   ├── session.dart       # OcSessionData (freezed), OurChatSession provider - chat session data
│   ├── event.dart         # OurChatEventSystem provider - real-time event streaming via gRPC
│   ├── server.dart        # OurChatServer - gRPC channel, TLS, interceptors
│   ├── database.dart      # Platform-conditional export (web vs desktop)
│   ├── database_desktop.dart  # drift tables + PublicOurChatDatabase / OurChatDatabase
│   ├── database_web.dart      # Web-specific database implementation
│   ├── chore.dart          # Utilities: OurChatTime, upload/download, MarkdownToText, safeRequest, AppStyles
│   ├── const.dart          # Constants: status codes, event types, permissions, ScreenMode enum
│   ├── log.dart            # Logger setup (platform-conditional)
│   ├── log_desktop.dart    # Desktop logger with file output
│   ├── log_web.dart        # Web logger (console only)
│   ├── ui.dart             # Shared UI helper (cardWithPadding)
│   └── stubs/              # Web stubs for desktop-only packages (window_manager, tray_manager, etc.)
├── service/               # Generated gRPC/protobuf Dart code (do not edit)
└── l10n/                  # Internationalization ARB files (en, zh)
```

### State Management (Riverpod)

The app uses **Riverpod with code generation** (`@riverpod` / `@Riverpod` annotations). All providers are generated by `build_runner`.

**Core Providers (keepAlive: true):**

| Provider | Type | Description |
|---|---|---|
| `configProvider` | `ConfigNotifier` → `OurChatConfig` | Global app configuration (server, theme, language, credentials) |
| `authProvider` | `AuthNotifier` → `AuthState` | Authentication state (login/register/logout, token management) |
| `thisAccountIdProvider` | `ThisAccountIdNotifier` → `Int64?` | Currently logged-in user's account ID |
| `ourChatServerProvider` | `OurChatServerNotifier` → `OurChatServer` | Active server connection (gRPC channel) |
| `ourChatEventSystemProvider` | `OurChatEventSystem` → `bool` | Real-time event listener (message streaming) |

**Parameterized Providers (keepAlive: true):**

| Provider | Type | Description |
|---|---|---|
| `ourChatAccountProvider(id)` | `OurChatAccount` → `AccountData` | Per-user account data with caching |
| `ourChatSessionProvider(id)` | `OurChatSession` → `OcSessionData` | Per-session data with caching |

**UI Providers:**

| Provider | Type | Description |
|---|---|---|
| `screenModeProvider` | `ScreenModeNotifier` → `ScreenMode` | Desktop/mobile layout detection |

**Pattern for adding a new provider:**

```dart
// In your file, use the @riverpod annotation:
@riverpod
class MyNotifier extends _$MyNotifier {
  @override
  MyState build() => MyState.initial();

  void update(MyState newState) => state = newState;
}

// Then run: dart run build_runner build --delete-conflicting-outputs
```

**Accessing providers in widgets:**

- Use `ConsumerWidget` or `ConsumerStatefulWidget` instead of `StatelessWidget`/`StatefulWidget`
- Read: `ref.read(provider)` — one-time read, no rebuild
- Watch: `ref.watch(provider)` — rebuild on change
- Notifier access: `ref.read(provider.notifier).someMethod()`

### Data Classes (freezed)

Immutable models use `@freezed` annotation with code generation:

```dart
@freezed
abstract class MyData with _$MyData {
  const factory MyData({required String field}) = _MyData;
  factory MyData.fromJson(Map<String, dynamic> json) => _$MyDataFromJson(json);
}
```

Key freezed models: `OurChatConfig`, `ServerConfig`, `LanguageConfig`, `AuthState`, `AccountData`, `OcSessionData`.

### Local Database (drift)

Two separate SQLite databases, with platform-conditional implementations:

**Public Database** (`PublicOurChatDatabase`) — shared across all accounts:
- `PublicAccount`: id, username, ocid, avatarKey, status, publicUpdateTime
- `PublicSession`: sessionId, name, avatarKey, createdTime, updatedTime, size, description

**Private Database** (`OurChatDatabase`) — per-user, named `OurChatDB_{accountId}`:
- `Account`: id, email, registerTime, updateTime, friendsJson, sessionsJson, latestMsgTime
- `Session`: sessionId, members (JSON), roles (JSON), myPermissions (JSON)
- `Record`: eventId, sessionId, eventType, sender, time, data (JSON), read

Platform implementations:
- Desktop/Mobile: `database_desktop.dart` — native SQLite via `drift_flutter`
- Web: `database_web.dart` — WASM-based SQLite

### Server Communication (gRPC)

`OurChatServer` manages the gRPC connection:
- Uses `GrpcOrGrpcWebClientChannel` for cross-platform support (gRPC native + gRPC-Web)
- `OurChatInterceptor` adds Bearer token authentication to all calls
- `OurChatServer.newStub()` creates an `OurChatServiceClient` with the interceptor attached
- TLS detection via `OurChatServer.tlsEnabled()` (tries TLS first, falls back to insecure)
- `safeRequest()` wrapper handles rate limiting (429 → retry) and error display

### Event System

`OurChatEventSystem` is a Riverpod notifier that:
- Connects to `fetchMsgs` gRPC streaming endpoint
- Processes incoming events by type: `msg`, `newFriendInvitationNotification`, `friendInvitationResultNotification`
- Saves events to local database (`Record` table)
- Supports listener registration for UI updates
- Auto-reconnects on disconnect (3-second delay)

### Caching Strategy

Account and session data use a 5-minute cache freshness check:
1. Check local DB for existing data
2. Compare server's `updatedTime` with local cache
3. Only fetch full data if stale or missing
4. Update both local DB and Riverpod state

### UI Structure

- `MainApp` — root `ConsumerStatefulWidget`, initializes window/tray on desktop, routes to `AutoLogin` or `ServerSetting`
- `AutoLogin` — attempts login with saved credentials
- `Home` — main shell with 4 tabs: Sessions, Friends, Settings, Me (User)
- Desktop: `NavigationRail` on left + content area; Mobile: `BottomNavigationBar`
- `ScreenMode` enum (`desktop`/`mobile`) determined by aspect ratio (width > height = desktop)
- `AppStyles` class centralizes spacing, font sizes, and icon sizes

### Navigation

Uses `Navigator.push`/`Navigator.pushReplacement` with `MaterialPageRoute` (not named routes).

## Development Workflow

### Code Generation

**After modifying Riverpod providers** (files with `@riverpod` / `@Riverpod` annotations):

```bash
dart run build_runner build --delete-conflicting-outputs
```

This regenerates `*.g.dart` files for providers, database, freezed models, and JSON serialization.

**When protobuf definitions change** (`service/` directory):

```bash
python script/generate.pb.dart.py
```

**When database schema changes** (`lib/core/database_desktop.dart` or `database_web.dart`):

```bash
dart run build_runner build --delete-conflicting-outputs
```

### Testing

- Unit tests can be added in `test/` directory.
- Integration tests in `integration_test/`.
- Run with `flutter test`.

### Code Quality

- Uses `flutter_lints` for static analysis.
- Format code with `dart format`.
- Follow Flutter best practices and the existing project patterns.

## Common Development Tasks

### Adding a New UI Screen

1. Create a new Dart file in `lib/` (e.g., `new_screen.dart`).
2. Use `ConsumerWidget` or `ConsumerStatefulWidget` (not plain `StatelessWidget`).
3. Add navigation using `Navigator.push(context, MaterialPageRoute(...))`.
4. Access state via `ref.watch()` / `ref.read()`.
5. Add translations in ARB files if text needs localization.

### Adding a New Riverpod Provider

1. Create a new Dart file or add to an existing one in `lib/core/`.
2. Define the state class with `@freezed` if it's a complex model.
3. Define the notifier with `@riverpod` or `@Riverpod(keepAlive: true)` annotation.
4. Run `dart run build_runner build --delete-conflicting-outputs`.
5. Access in widgets via `ref.watch(provider)` / `ref.read(provider)`.

### Integrating a New gRPC Service

1. Ensure the `.proto` file is in `service/` directory (shared with server).
2. Regenerate Dart code with `python script/generate.pb.dart.py`.
3. Use `ref.read(ourChatServerProvider).newStub()` to create a service client.
4. Wrap calls with `safeRequest()` for error handling.
5. Update UI to use the new functionality.

### Updating Local Database Schema

1. Modify table definitions in `lib/core/database_desktop.dart` (and `database_web.dart` if needed).
2. Run `dart run build_runner build --delete-conflicting-outputs`.
3. Update any queries that depend on the changed schema.
4. Handle migrations if needed (drift provides migration support).

### Adding Internationalization

1. Add new keys to `lib/l10n/app_en.arb` (and other language files).
2. Run `flutter gen-l10n` to regenerate localization classes.
3. Use `AppLocalizations.of(context)!.keyName` in widgets.

### Desktop-Specific Features

- Use `kIsWeb` to guard web-incompatible code (not `Platform.isX` — it throws on web).
- Desktop-only imports use conditional stubs: `import 'package:X' if (dart.library.html) 'package:ourchat/core/stubs/X_stub.dart'`.
- `window_manager` for window control (minimize, close, etc.).
- `tray_manager` for system tray icons and menus.

## Troubleshooting

### Connection Issues

- **Server unreachable**: Check server address/port in settings, verify server is running
- **TLS errors**: Ensure server TLS certificate is valid, or disable TLS for development
- **gRPC errors**: Verify protobuf definitions match between client and server

### Build Issues

- **Flutter version mismatch**: Check `pubspec.yaml` for Dart SDK constraints
- **Missing native dependencies**: Install platform-specific build tools
- **Code generation failures**: Run `dart run build_runner build --delete-conflicting-outputs`

### Runtime Issues

- **Database errors**: Delete `~/.local/share/ourchat/` (Linux) or app data directory to reset local database
- **UI rendering problems**: Check Flutter channel compatibility, try `flutter clean`
- **Internationalization missing**: Run `flutter gen-l10n` to regenerate localization files

### Platform-Specific Issues

- **Linux tray not appearing**: Install `libappindicator` or `libayatana-appindicator`
- **Windows window management**: Ensure `window_manager` permissions in manifest
- **macOS permissions**: Grant necessary entitlements for network, tray, etc.
- **Web gRPC**: Uses `GrpcOrGrpcWebClientChannel` which falls back to gRPC-Web

## Important Notes

- The client is **cross-platform**; use `kIsWeb` guard for web-incompatible code, not `Platform.isX`.
- **State management** uses Riverpod with code generation; use `ConsumerWidget`/`ConsumerStatefulWidget` for widgets that need providers.
- **Database operations** are async; use `await` and handle errors appropriately.
- **gRPC calls** should use `safeRequest()` wrapper for rate limiting and error display.
- **UI responsiveness**: The app adapts to mobile/desktop via `ScreenMode` based on aspect ratio.
- **Internationalization**: All user-facing strings should be localized via ARB files.
- **Assets**: Image assets are in `assets/images/`; update `pubspec.yaml` if adding new asset directories.
- **Code generation**: Generated files (`*.pb.dart`, `*.g.dart`, `*.freezed.dart`) should not be edited manually.
- **Global state**: `publicDB`, `privateDB`, `l10n`, `logger` are top-level globals defined in `main.dart`.
