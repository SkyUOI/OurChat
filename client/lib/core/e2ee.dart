import 'dart:convert';
import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/crypto.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/secret_store.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'e2ee.g.dart';

/// Payload encrypted under a session's room key.
class EncryptedPayload {
  final String markdownText;
  final List<String> involvedFiles;

  const EncryptedPayload({
    required this.markdownText,
    required this.involvedFiles,
  });

  String toJson() => jsonEncode({'m': markdownText, 'f': involvedFiles});

  static EncryptedPayload fromJson(String s) {
    final m = jsonDecode(s) as Map<String, dynamic>;
    return EncryptedPayload(
      markdownText: m['m'] as String? ?? '',
      involvedFiles:
          (m['f'] as List?)?.map((e) => e.toString()).toList() ?? const [],
    );
  }
}

/// In-memory + persistent store of per-session E2EE room keys, plus helpers to
/// encrypt/decrypt message payloads.
///
/// Scoped to one (serverId, accountId): each logged-in identity keeps its own
/// room keys, persisted under its own SecretStore namespace.
@Riverpod(keepAlive: true)
class E2eeStore extends _$E2eeStore {
  /// sessionId (Int64.toInt()) -> room key bytes
  final Map<int, Uint8List> _keys = {};

  /// This store's server id (family argument).
  String get _serverId => serverId;

  /// This store's account id (family argument).
  Int64 get _accountId => accountId;

  /// This identity's RSA private key, lazily loaded from SecretStore.
  Uint8List? _privateKey;

  @override
  Map<int, Uint8List> build(String serverId, Int64 accountId) => _keys;

  /// Whether we hold a room key for [sessionId] (i.e. messages to/from it are
  /// end-to-end encrypted for us).
  bool hasKey(Int64 sessionId) => _keys.containsKey(sessionId.toInt());

  /// The room key for [sessionId], or null.
  Uint8List? keyFor(Int64 sessionId) => _keys[sessionId.toInt()];

  /// Store a freshly received/generated room key, persisting it securely.
  Future<void> storeKey(Int64 sessionId, Uint8List key) async {
    _keys[sessionId.toInt()] = key;
    await SecretStore.saveRoomKey(_serverId, sessionId, key);
    logger.i('E2EE: stored room key for session $sessionId');
  }

  /// Drop a session's room key (e.g. when the session is dee2eeized).
  Future<void> removeKey(Int64 sessionId) async {
    _keys.remove(sessionId.toInt());
    await SecretStore.deleteRoomKey(_serverId, sessionId);
  }

  /// Load any persisted room keys on demand for a session.
  Future<Uint8List?> loadKey(Int64 sessionId) async {
    if (_keys.containsKey(sessionId.toInt())) {
      return _keys[sessionId.toInt()];
    }
    final k = await SecretStore.readRoomKey(_serverId, sessionId);
    if (k != null) {
      _keys[sessionId.toInt()] = k;
    }
    return k;
  }

  /// Encrypt a message payload for [sessionId] under its room key.
  ///
  /// Returns the base64 ciphertext string to place in the `markdown_text`
  /// wire field, with `is_encrypted = true`. Throws if no room key is held.
  String encryptMessage(Int64 sessionId, EncryptedPayload payload) {
    final key = _keys[sessionId.toInt()];
    if (key == null) {
      throw StateError('no room key for session $sessionId');
    }
    final plaintext = Uint8List.fromList(utf8.encode(payload.toJson()));
    final ct = aesGcmEncrypt(key, plaintext);
    return base64.encode(ct);
  }

  /// Decrypt a base64 ciphertext string received in an encrypted message.
  /// Returns the original payload, or null on any failure (wrong/missing key,
  /// tampered ciphertext).
  Future<EncryptedPayload?> decryptMessage(Int64 sessionId, String b64) async {
    var key = _keys[sessionId.toInt()];
    key ??= await loadKey(sessionId);
    if (key == null) {
      logger.w('E2EE: cannot decrypt, no room key for session $sessionId');
      return null;
    }
    try {
      final packed = base64.decode(b64);
      final plain = aesGcmDecrypt(key, packed);
      return EncryptedPayload.fromJson(utf8.decode(plain));
    } catch (e) {
      logger.w('E2EE: failed to decrypt message for session $sessionId: $e');
      return null;
    }
  }

  /// Wrap a room key for a recipient whose RSA public key (PKCS#1 DER) is
  /// [derPublicKey]. Used by the e2eeize initiator to distribute the room key.
  Uint8List wrapRoomKey(Uint8List roomKey, Uint8List derPublicKey) {
    return rsaEncrypt(derPublicKey, roomKey);
  }

  /// This identity's private key, loaded (and cached) from SecretStore.
  Future<Uint8List?> _loadPrivateKey() async {
    if (_privateKey != null) return _privateKey;
    _privateKey = await SecretStore.readPrivateKey(_serverId, _accountId);
    return _privateKey;
  }

  /// Unwrap (decrypt) a room key that was wrapped for us, using this
  /// identity's own private key.
  Future<Uint8List?> unwrapRoomKey(Uint8List wrapped) async {
    final priv = await _loadPrivateKey();
    if (priv == null) {
      logger.w('E2EE: cannot unwrap room key, private key not loaded');
      return null;
    }
    try {
      return rsaDecrypt(priv, wrapped);
    } catch (e) {
      logger.w('E2EE: failed to unwrap room key: $e');
      return null;
    }
  }
}
