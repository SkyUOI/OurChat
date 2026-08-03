import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:ourchat/core/crypto.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/secret_store.dart';
import 'package:ourchat/core/server.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/auth/authorize/v1/authorize.pb.dart';
import 'package:ourchat/service/auth/register/v1/register.pb.dart';
import 'package:ourchat/service/auth/v1/auth.pbgrpc.dart';

part 'auth_notifier.freezed.dart';
part 'auth_notifier.g.dart';

/// Authentication state
@freezed
abstract class AuthState with _$AuthState {
  const factory AuthState({
    @Default(false) bool isLoading,
    String? error,
    String? serverId,
    Int64? accountId,
    String? token,
    String? ocid,
  }) = _AuthState;

  const AuthState._();

  bool get isAuthenticated => accountId != null && token != null;
}

/// Authentication notifier. Responsible ONLY for the auth RPC, setting the
/// bearer token on the target server's interceptor, and loading the E2EE
/// private key. It does NOT open the private database or build a
/// [OurChatInstance] — that is the caller's job (see `_handleAuthSuccess` in
/// `auth.dart`, `AutoLogin` in `main.dart`, `LiveClientFixture`).
///
/// `serverId` (= `OurChatServer.uniqueIdentifier`) is carried in [AuthState]
/// so downstream code knows which server's namespace the token/private-key
/// belong to.
@Riverpod(keepAlive: true)
class AuthNotifier extends _$AuthNotifier {
  /// In-memory cache of the logged-in account's RSA private key, used for E2EE
  /// decryption. Persisted in [SecretStore] keyed by (serverId, accountId).
  Uint8List? _privateKey;

  /// The pending private key from a registration that hasn't been persisted
  /// yet (the account id is only known after register() succeeds).
  Uint8List? _pendingPrivateKey;

  @override
  AuthState build() {
    return AuthState();
  }

  /// The currently loaded RSA private key (DER PKCS#1), or null if none.
  Uint8List? get privateKey => _privateKey;

  /// Login with an email or OCID and a password.
  ///
  /// [server] overrides the target server (defaults to the active
  /// `ourChatServerProvider`). The server must already have had
  /// `getServerInfo()` called so its `uniqueIdentifier` is populated.
  Future<bool> login({
    required String password,
    String? ocid,
    String? email,
    OurChatServer? server,
  }) async {
    if (ocid == null && email == null) {
      state = state.copyWith(error: l10n.emailOrOcidRequired);
      return false;
    }

    state = state.copyWith(isLoading: true, error: null);
    try {
      final OurChatServer srv = server ?? ref.read(ourChatServerProvider);
      final serverId = srv.uniqueIdentifier;
      if (serverId == null || serverId.isEmpty) {
        state = state.copyWith(
          isLoading: false,
          error: l10n.serverNotIdentified,
        );
        logger.e('Login failed: server uniqueIdentifier not ready');
        return false;
      }
      final channel = srv.channel;
      final authClient = AuthServiceClient(channel);

      final request = AuthRequest(password: password);
      if (email != null) {
        request.email = email;
      } else {
        request.ocid = ocid!;
      }

      final response = await authClient.auth(request);
      logger.i(
        'Login successful, user ID: ${response.id}, OCID: ${response.ocid}',
      );

      // Update the token on this server's interceptor
      srv.interceptor ??= OurChatInterceptor();
      srv.interceptor!.setToken(response.token);

      // Update state
      state = state.copyWith(
        isLoading: false,
        serverId: serverId,
        accountId: response.id,
        token: response.token,
        ocid: response.ocid,
      );

      // Update the current account id in app state (UI still reads via thisAccountIdProvider)
      ref.read(thisAccountIdProvider.notifier).setAccountId(response.id);

      // Load this account's E2EE private key from secure storage.
      await loadPrivateKey(response.id);

      return true;
    } on GrpcError catch (e) {
      final errorMessage = _handleAuthError(e);
      state = state.copyWith(isLoading: false, error: errorMessage);
      logger.e('Login failed: $errorMessage');
      return false;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: '${l10n.loginFailed}: $e',
      );
      logger.e('Login failed: $e');
      return false;
    }
  }

  /// Register with an email, password and username
  Future<bool> register({
    required String email,
    required String password,
    required String username,
    List<int>? publicKey,
    OurChatServer? server,
  }) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final OurChatServer srv = server ?? ref.read(ourChatServerProvider);
      final serverId = srv.uniqueIdentifier;
      if (serverId == null || serverId.isEmpty) {
        state = state.copyWith(
          isLoading: false,
          error: l10n.serverNotIdentified,
        );
        logger.e('Registration failed: server uniqueIdentifier not ready');
        return false;
      }
      final channel = srv.channel;
      final authClient = AuthServiceClient(channel);

      // Generate RSA key pair on registration. The public key is sent to the
      // server; the private key is captured locally and persisted to
      // SecretStore once we know the account id (see persistPrivateKey).
      var keyBytes = publicKey ?? <int>[];
      if (keyBytes.isEmpty) {
        try {
          final keyPair = generateRsaKeyPair();
          keyBytes = keyPair.publicKey;
          _pendingPrivateKey = keyPair.privateKey;
          logger.i('Generated new RSA key pair for registration');
        } catch (e) {
          logger.e('Failed to generate RSA key pair: $e');
        }
      }

      final request = RegisterRequest(
        email: email,
        password: password,
        name: username,
        publicKey: keyBytes,
      );

      final response = await authClient.register(request);
      logger.i(
        'Registration successful, user ID: ${response.id}, OCID: ${response.ocid}',
      );

      srv.interceptor ??= OurChatInterceptor();
      srv.interceptor!.setToken(response.token);

      state = state.copyWith(
        isLoading: false,
        serverId: serverId,
        accountId: response.id,
        token: response.token,
        ocid: response.ocid,
      );

      ref.read(thisAccountIdProvider.notifier).setAccountId(response.id);

      return true;
    } on GrpcError catch (e) {
      final errorMessage = _handleAuthError(e);
      state = state.copyWith(isLoading: false, error: errorMessage);
      logger.e('Registration failed: $errorMessage');
      return false;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: '${l10n.registerFailed}: $e',
      );
      logger.e('Registration failed: $e');
      return false;
    }
  }

  /// Allow the UI to hand over a private key it generated alongside the
  /// public key used for registration.
  void setPendingPrivateKey(Uint8List privateKey) {
    _pendingPrivateKey = privateKey;
  }

  /// Persist the pending private key (from registration) for [accountId] to
  /// secure storage and load it into memory.
  Future<void> persistPrivateKey(Int64 accountId) async {
    final serverId = state.serverId;
    if (_pendingPrivateKey != null) {
      if (serverId != null) {
        await SecretStore.savePrivateKey(
          serverId,
          accountId,
          _pendingPrivateKey!,
        );
      }
      _privateKey = _pendingPrivateKey;
      _pendingPrivateKey = null;
      logger.i('E2EE private key persisted for account $accountId');
    } else {
      await loadPrivateKey(accountId);
    }
  }

  /// Load the private key for [accountId] from secure storage into memory.
  Future<void> loadPrivateKey(Int64 accountId) async {
    final serverId = state.serverId;
    if (serverId == null) {
      logger.w('loadPrivateKey: no serverId in auth state');
      _privateKey = null;
      return;
    }
    _privateKey = await SecretStore.readPrivateKey(serverId, accountId);
    if (_privateKey == null) {
      logger.w('No E2EE private key found for account $accountId');
    }
  }

  /// Log out the current account's auth state (does not close the instance - the caller's job).
  void logout() {
    final srv = ref.read(ourChatServerProvider);
    if (srv.interceptor != null) {
      srv.interceptor!.setToken('');
    }

    state = AuthState();
    _privateKey = null;
    _pendingPrivateKey = null;

    ref.read(thisAccountIdProvider.notifier).clear();

    logger.i('User logged out');
  }

  /// Handle auth-related gRPC errors
  String _handleAuthError(GrpcError e) {
    switch (e.code) {
      case 5: // NOT_FOUND
        return l10n.userNotFound;
      case 7: // PERMISSION_DENIED
        return l10n.wrongPassword;
      case 3: // INVALID_ARGUMENT
        return l10n.invalidRequestParameters;
      case 6: // ALREADY_EXISTS (on registration)
        return l10n.emailOrUsernameExists;
      default:
        return e.message ?? l10n.authFailedCode(e.code);
    }
  }
}
