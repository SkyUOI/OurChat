import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/crypto.dart';

void main() {
  group('RSA keypair + OAEP round trip', () {
    test('encrypt then decrypt returns the original plaintext', () {
      final pair = generateRsaKeyPair();
      final message = Uint8List.fromList(utf8.encode('hello-e2ee-room-key'));
      final wrapped = rsaEncrypt(pair.publicKey, message);
      final unwrapped = rsaDecrypt(pair.privateKey, wrapped);
      expect(unwrapped, equals(message));
    });

    test('decoding a private key reproduces the same key material', () {
      final pair = generateRsaKeyPair();
      final decoded = decodeRsaPrivateKey(pair.privateKey);
      // The modulus is the canonical identifier of an RSA key.
      expect(decoded.modulus, isNotNull);
      expect(decoded.privateExponent, isNotNull);
    });

    test('decoding a public key reproduces the modulus', () {
      final pair = generateRsaKeyPair();
      final decoded = decodeRsaPublicKey(pair.publicKey);
      expect(decoded.modulus, isNotNull);
    });
  });

  group('AES-256-GCM', () {
    test('decrypt reverses encrypt', () {
      final key = generateRoomKey();
      expect(key.length, 32);
      final plaintext = Uint8List.fromList(
        utf8.encode('a secret chat message with ünïcode ✓'),
      );
      final packed = aesGcmEncrypt(key, plaintext);
      // iv(12) + ciphertext + tag(16)
      expect(packed.length, 12 + plaintext.length + 16);
      expect(aesGcmDecrypt(key, packed), equals(plaintext));
    });

    test('tampering with the ciphertext throws', () {
      final key = generateRoomKey();
      final packed = aesGcmEncrypt(key, Uint8List.fromList(utf8.encode('msg')));
      packed[packed.length - 1] ^= 0xFF; // flip a tag bit
      expect(() => aesGcmDecrypt(key, packed), throwsA(anything));
    });

    test('wrong key fails authentication', () {
      final key = generateRoomKey();
      final other = generateRoomKey();
      final packed = aesGcmEncrypt(key, Uint8List.fromList(utf8.encode('msg')));
      expect(() => aesGcmDecrypt(other, packed), throwsA(anything));
    });

    test('random IV: two encryptions of the same plaintext differ', () {
      final key = generateRoomKey();
      final plaintext = Uint8List.fromList(utf8.encode('msg'));
      final a = aesGcmEncrypt(key, plaintext);
      final b = aesGcmEncrypt(key, plaintext);
      expect(a, isNot(equals(b)));
    });
  });
}
