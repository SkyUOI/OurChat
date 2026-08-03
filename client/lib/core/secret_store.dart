import 'dart:convert';
import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ourchat/core/log.dart';

/// Persistent, platform-backed secret storage.
///
/// All secrets (saved login password, E2EE RSA private keys, E2EE room keys)
/// are stored here instead of [SharedPreferences], which is unencrypted on
/// every platform. Values are kept out of plain config files and out of logs.
///
/// Every key is namespaced by the server's [uniqueIdentifier] (the stable
/// server id from `getServerInfo`), so that the same account id / session id
/// on two different servers never collide.
///
/// * Android  -> AndroidKeyStore
/// * iOS/macOS -> Keychain
/// * Linux     -> libsecret
/// * Windows   -> DPAPI
/// * Web       -> WebCrypto (best-effort; web is inherently less secure)
class SecretStore {
  SecretStore._();

  static const _prefix = 'ourchat:';
  static const _iosOptions = IOSOptions(
    accessibility: KeychainAccessibility.first_unlock,
  );
  static const _aOptions = AndroidOptions();

  static final FlutterSecureStorage _storage = FlutterSecureStorage(
    iOptions: _iosOptions,
    aOptions: _aOptions,
  );

  // ── account credentials (password) ──

  static String _credKey(String serverId, String account) =>
      '${_prefix}cred:$serverId:$account';

  /// Persist the password for [account] (the email or OCID the user logs in
  /// with) on [serverId]. Pass an empty [password] to clear a previously saved
  /// password.
  static Future<void> saveCredential(
    String serverId,
    String account,
    String password,
  ) async {
    try {
      if (password.isEmpty) {
        await _storage.delete(key: _credKey(serverId, account));
      } else {
        await _storage.write(key: _credKey(serverId, account), value: password);
      }
    } catch (e) {
      logger.e('SecretStore: failed to save credential: $e');
    }
  }

  static Future<String?> readCredential(String serverId, String account) async {
    try {
      return await _storage.read(key: _credKey(serverId, account));
    } catch (e) {
      logger.e('SecretStore: failed to read credential: $e');
      return null;
    }
  }

  static Future<void> deleteCredential(String serverId, String account) async {
    try {
      await _storage.delete(key: _credKey(serverId, account));
    } catch (e) {
      logger.e('SecretStore: failed to delete credential: $e');
    }
  }

  // ── E2EE RSA private keys (per account on a server) ──

  static String _privKeyKey(String serverId, Int64 accountId) =>
      '${_prefix}privkey:$serverId:${accountId.toInt()}';

  static Future<void> savePrivateKey(
    String serverId,
    Int64 accountId,
    Uint8List privateKey,
  ) async {
    try {
      await _storage.write(
        key: _privKeyKey(serverId, accountId),
        value: base64.encode(privateKey),
      );
    } catch (e) {
      logger.e('SecretStore: failed to save private key: $e');
    }
  }

  static Future<Uint8List?> readPrivateKey(
    String serverId,
    Int64 accountId,
  ) async {
    try {
      final v = await _storage.read(key: _privKeyKey(serverId, accountId));
      if (v == null || v.isEmpty) return null;
      return base64.decode(v);
    } catch (e) {
      logger.e('SecretStore: failed to read private key: $e');
      return null;
    }
  }

  // ── E2EE room keys (per session on a server) ──

  static String _roomKeyKey(String serverId, Int64 sessionId) =>
      '${_prefix}roomkey:$serverId:${sessionId.toInt()}';

  static Future<void> saveRoomKey(
    String serverId,
    Int64 sessionId,
    Uint8List key,
  ) async {
    try {
      await _storage.write(
        key: _roomKeyKey(serverId, sessionId),
        value: base64.encode(key),
      );
    } catch (e) {
      logger.e('SecretStore: failed to save room key: $e');
    }
  }

  static Future<Uint8List?> readRoomKey(
    String serverId,
    Int64 sessionId,
  ) async {
    try {
      final v = await _storage.read(key: _roomKeyKey(serverId, sessionId));
      if (v == null || v.isEmpty) return null;
      return base64.decode(v);
    } catch (e) {
      logger.e('SecretStore: failed to read room key: $e');
      return null;
    }
  }

  static Future<void> deleteRoomKey(String serverId, Int64 sessionId) async {
    try {
      await _storage.delete(key: _roomKeyKey(serverId, sessionId));
    } catch (e) {
      logger.e('SecretStore: failed to delete room key: $e');
    }
  }

  /// Remove every secret managed by this app. Called on explicit logout /
  /// "clear local data" flows. Best-effort: errors are logged, not thrown.
  static Future<void> clearAll() async {
    try {
      final all = await _storage.readAll();
      for (final key in all.keys) {
        if (key.startsWith(_prefix)) {
          await _storage.delete(key: key);
        }
      }
    } catch (e) {
      logger.e('SecretStore: failed to clear all: $e');
    }
  }
}
