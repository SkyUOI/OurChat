import 'dart:convert';
import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/crypto.dart';
import 'package:ourchat/core/e2ee.dart';

/// Integration tests for `E2eeStore` — the production Riverpod notifier that
/// owns per-session room keys and performs message encrypt/decrypt.
///
/// These exercise the *real* `encryptMessage`/`decryptMessage`/`storeKey` code
/// paths (not just the underlying crypto primitives), guaranteeing that the
/// base64 + JSON-payload + AES-GCM composition used by `event.dart` round-trips
/// correctly.
void main() {
  late ProviderContainer container;
  late E2eeStore store;

  setUp(() {
    container = ProviderContainer();
    store = container.read(e2eeStoreProvider('test', Int64(1)).notifier);
  });

  tearDown(() => container.dispose());

  test('storeKey / hasKey / keyFor manage in-memory keys', () async {
    final sessionId = Int64(9001);
    expect(store.hasKey(sessionId), isFalse);
    expect(store.keyFor(sessionId), isNull);

    final key = Uint8List.fromList(List.filled(32, 7));
    await store.storeKey(sessionId, key);

    expect(store.hasKey(sessionId), isTrue);
    expect(store.keyFor(sessionId), equals(key));
  });

  test(
    'encryptMessage -> decryptMessage recovers the original payload',
    () async {
      final sessionId = Int64(42);
      await store.storeKey(sessionId, Uint8List.fromList(List.filled(32, 9)));

      const payload = EncryptedPayload(
        markdownText: 'hello **encrypted** world 🦀',
        involvedFiles: ['filekey1', 'filekey2'],
      );

      final ciphertext = store.encryptMessage(sessionId, payload);
      // The wire form is base64 text, never the plaintext.
      expect(ciphertext, isNot(contains('hello')));
      expect(() => base64.decode(ciphertext), returnsNormally);

      final recovered = await store.decryptMessage(sessionId, ciphertext);
      expect(recovered, isNotNull);
      expect(recovered!.markdownText, payload.markdownText);
      expect(recovered.involvedFiles, payload.involvedFiles);
    },
  );

  test('encryptMessage without a room key throws StateError', () {
    final sessionId = Int64(9999); // no key stored
    expect(
      () => store.encryptMessage(
        sessionId,
        const EncryptedPayload(markdownText: 'x', involvedFiles: []),
      ),
      throwsA(isA<StateError>()),
    );
  });

  test('decryptMessage returns null for a session without a key', () async {
    final sessionId = Int64(1234); // no key
    final result = await store.decryptMessage(
      sessionId,
      'dGhpcyBpcyBub3QgdmFsaWQ=',
    );
    expect(result, isNull);
  });

  test('decryptMessage returns null when the ciphertext is tampered', () async {
    final sessionId = Int64(7);
    await store.storeKey(sessionId, Uint8List.fromList(List.filled(32, 1)));

    final ciphertext = store.encryptMessage(
      sessionId,
      const EncryptedPayload(markdownText: 'secret', involvedFiles: []),
    );
    // Flip the last byte (corrupts the GCM auth tag) -> must fail closed.
    final bytes = base64.decode(ciphertext);
    bytes[bytes.length - 1] ^= 0xFF;
    final tampered = base64.encode(bytes);

    final result = await store.decryptMessage(sessionId, tampered);
    expect(result, isNull, reason: 'tampered ciphertext must not decrypt');
  });

  test('decryptMessage returns null for malformed base64', () async {
    final sessionId = Int64(8);
    await store.storeKey(sessionId, Uint8List.fromList(List.filled(32, 2)));
    final result = await store.decryptMessage(sessionId, '!!!not-base64!!!');
    expect(result, isNull);
  });

  test(
    'two payloads under the same key produce different ciphertexts',
    () async {
      final sessionId = Int64(100);
      await store.storeKey(sessionId, Uint8List.fromList(List.filled(32, 5)));
      final a = store.encryptMessage(
        sessionId,
        const EncryptedPayload(markdownText: 'same', involvedFiles: []),
      );
      final b = store.encryptMessage(
        sessionId,
        const EncryptedPayload(markdownText: 'same', involvedFiles: []),
      );
      // Random IV per encryption => distinct ciphertexts.
      expect(a, isNot(equals(b)));
    },
  );

  test('wrapRoomKey produces RSA ciphertext decryptable to the original', () {
    // Simulates the e2eeize initiator wrapping the room key for a member whose
    // public key is known.
    final pair = generateRsaKeyPair();
    final roomKey = Uint8List.fromList(List.filled(32, 0xAB));
    final wrapped = store.wrapRoomKey(roomKey, pair.publicKey);
    final unwrapped = rsaDecrypt(pair.privateKey, wrapped);
    expect(unwrapped, equals(roomKey));
  });
}
