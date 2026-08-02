import 'dart:math';
import 'dart:typed_data';
import 'package:pointycastle/asn1.dart';
import 'package:pointycastle/export.dart';

/// Symmetric room-key length (AES-256).
const int roomKeyLength = 32;

/// AES-GCM initialization-vector length.
const int _gcmIvLength = 12;

/// AES-GCM authentication-tag length.
const int _gcmTagLength = 16;

/// Generates a 2048-bit RSA key pair.
///
/// Returns a tuple of (publicKey as PKCS#1 DER bytes, privateKey as PKCS#1 DER bytes).
/// The public key is sent to the server during registration.
/// The private key is stored locally and used for E2EE operations.
({Uint8List publicKey, Uint8List privateKey}) generateRsaKeyPair() {
  final keyGen = RSAKeyGenerator()
    ..init(
      ParametersWithRandom(
        RSAKeyGeneratorParameters(
          BigInt.parse('65537'), // e — common public exponent
          2048, // bit length
          64, // certainty
        ),
        _fortunaRandom(),
      ),
    );

  final pair = keyGen.generateKeyPair();
  final publicKey = pair.publicKey;
  final privateKey = pair.privateKey;

  // Encode public key as PKCS#1 DER (RSAPublicKey structure)
  final publicKeyBytes = _encodeRsaPublicKeyToPkcs1Der(publicKey);

  // Encode private key as PKCS#1 DER (RSAPrivateKey structure)
  final privateKeyBytes = _encodeRsaPrivateKeyToPkcs1Der(privateKey);

  return (publicKey: publicKeyBytes, privateKey: privateKeyBytes);
}

// ── Fortuna-based CSPRNG ──

FortunaRandom _fortunaRandom() {
  final random = Random.secure();
  final seed = Uint8List(32);
  for (var i = 0; i < 32; i++) {
    seed[i] = random.nextInt(256);
  }
  final prng = FortunaRandom()..seed(KeyParameter(seed));
  return prng;
}

// ── PKCS#1 DER encoding helpers ──

/// Encodes an RSA public key to PKCS#1 DER format:
///   RSAPublicKey ::= SEQUENCE {
///     modulus           INTEGER,
///     publicExponent    INTEGER
///   }
Uint8List _encodeRsaPublicKeyToPkcs1Der(RSAPublicKey key) {
  final modulus = _encodeInteger(key.modulus!);
  final exponent = _encodeInteger(key.publicExponent!);
  final seq = _encodeSequence(Uint8List.fromList([...modulus, ...exponent]));
  return seq;
}

/// Encodes an RSA private key to PKCS#1 DER format:
///   RSAPrivateKey ::= SEQUENCE {
///     version           INTEGER (0),
///     modulus           INTEGER,
///     publicExponent    INTEGER,
///     privateExponent   INTEGER,
///     prime1            INTEGER,
///     prime2            INTEGER,
///     exponent1         INTEGER,
///     exponent2         INTEGER,
///     coefficient       INTEGER
///   }
Uint8List _encodeRsaPrivateKeyToPkcs1Der(RSAPrivateKey key) {
  final version = _encodeInteger(BigInt.zero);
  final modulus = _encodeInteger(key.modulus!);
  final publicExponent = _encodeInteger(key.publicExponent!);
  final privateExponent = _encodeInteger(key.privateExponent!);
  final p = _encodeInteger(key.p!);
  final q = _encodeInteger(key.q!);
  final dP = _encodeInteger(key.privateExponent! % (key.p! - BigInt.one));
  final dQ = _encodeInteger(key.privateExponent! % (key.q! - BigInt.one));
  final qInv = _encodeInteger(key.q!.modInverse(key.p!));

  return _encodeSequence(
    Uint8List.fromList([
      ...version,
      ...modulus,
      ...publicExponent,
      ...privateExponent,
      ...p,
      ...q,
      ...dP,
      ...dQ,
      ...qInv,
    ]),
  );
}

// ── ASN.1 DER primitives ──

Uint8List _encodeLength(int length) {
  if (length < 128) {
    return Uint8List.fromList([length]);
  }
  final bytes = <int>[];
  var l = length;
  while (l > 0) {
    bytes.insert(0, l & 0xFF);
    l >>= 8;
  }
  return Uint8List.fromList([0x80 | bytes.length, ...bytes]);
}

Uint8List _encodeInteger(BigInt value) {
  if (value == BigInt.zero) {
    return Uint8List.fromList([0x02, 0x01, 0x00]);
  }
  var bytes = value.toRadixString(16);
  if (bytes.length % 2 != 0) {
    bytes = '0$bytes';
  }
  final hexBytes = Uint8List.fromList(
    List.generate(
      bytes.length ~/ 2,
      (i) => int.parse(bytes.substring(i * 2, i * 2 + 2), radix: 16),
    ),
  );
  // Add leading zero byte if high bit is set (to keep value positive)
  final withSign = (hexBytes[0] & 0x80) != 0
      ? Uint8List.fromList([0x00, ...hexBytes])
      : hexBytes;
  final len = _encodeLength(withSign.length);
  return Uint8List.fromList([0x02, ...len, ...withSign]);
}

Uint8List _encodeSequence(Uint8List contents) {
  final len = _encodeLength(contents.length);
  return Uint8List.fromList([0x30, ...len, ...contents]);
}

// ──────────────────────────────────────────────────────────────────────────
// E2EE primitives: key decoding, RSA-OAEP key wrap, AES-GCM message encryption
// ──────────────────────────────────────────────────────────────────────────

/// Decodes a PKCS#1 DER RSA private key into an [RSAPrivateKey].
RSAPrivateKey decodeRsaPrivateKey(Uint8List der) {
  final seq = ASN1Parser(der).nextObject() as ASN1Sequence;
  final ints = seq.elements!.cast<ASN1Integer>();
  // PKCS#1 RSAPrivateKey layout:
  // version, modulus, publicExponent, privateExponent, prime1, prime2,
  // exponent1, exponent2, coefficient
  // RSAPrivateKey(modulus, privateExponent, p, q) — public exponent is derived.
  return RSAPrivateKey(
    ints[1].integer!,
    ints[3].integer!,
    ints[4].integer!,
    ints[5].integer!,
  );
}

/// Decodes a PKCS#1 DER RSA public key into an [RSAPublicKey].
RSAPublicKey decodeRsaPublicKey(Uint8List der) {
  final seq = ASN1Parser(der).nextObject() as ASN1Sequence;
  final ints = seq.elements!.cast<ASN1Integer>();
  // PKCS#1 RSAPublicKey layout: modulus, publicExponent
  return RSAPublicKey(ints[0].integer!, ints[1].integer!);
}

/// Generates a fresh random symmetric room key.
Uint8List generateRoomKey() => _randomBytes(roomKeyLength);

/// RSA-OAEP (SHA-256) encrypt [plaintext] under [publicKey].
///
/// Used to wrap the per-session room key for each recipient. With RSA-2048 the
/// maximum plaintext size is 190 bytes, comfortably larger than the 32-byte
/// room key.
Uint8List rsaEncrypt(Uint8List derPublicKey, Uint8List plaintext) {
  final pub = decodeRsaPublicKey(derPublicKey);
  final engine = OAEPEncoding.withSHA256(RSAEngine())
    ..init(true, PublicKeyParameter<RSAPublicKey>(pub));
  return engine.process(plaintext);
}

/// RSA-OAEP (SHA-256) decrypt [ciphertext] with [derPrivateKey].
Uint8List rsaDecrypt(Uint8List derPrivateKey, Uint8List ciphertext) {
  final priv = decodeRsaPrivateKey(derPrivateKey);
  final engine = OAEPEncoding.withSHA256(RSAEngine())
    ..init(false, PrivateKeyParameter<RSAPrivateKey>(priv));
  return engine.process(ciphertext);
}

/// AES-256-GCM encryption of [plaintext].
///
/// Returns `iv(12) || ciphertext || tag(16)`. A fresh random IV is generated
/// per call. The caller must base64-encode the result for transport in a text
/// protobuf field.
Uint8List aesGcmEncrypt(Uint8List key, Uint8List plaintext) {
  if (key.length != roomKeyLength) {
    throw ArgumentError('AES key must be $roomKeyLength bytes');
  }
  final iv = _randomBytes(_gcmIvLength);
  final cipher = GCMBlockCipher(AESEngine())
    ..init(
      true,
      AEADParameters(KeyParameter(key), _gcmTagLength * 8, iv, Uint8List(0)),
    );
  final tmp = Uint8List(plaintext.length + _gcmTagLength);
  var n = cipher.processBytes(plaintext, 0, plaintext.length, tmp, 0);
  n += cipher.doFinal(tmp, n);
  return Uint8List.fromList([...iv, ...tmp.sublist(0, n)]);
}

/// AES-256-GCM decryption of `iv(12) || ciphertext || tag(16)`.
///
/// Throws if the authentication tag does not verify (tampered/ciphertext).
Uint8List aesGcmDecrypt(Uint8List key, Uint8List packed) {
  if (key.length != roomKeyLength) {
    throw ArgumentError('AES key must be $roomKeyLength bytes');
  }
  if (packed.length < _gcmIvLength + _gcmTagLength) {
    throw ArgumentError('ciphertext too short');
  }
  final iv = packed.sublist(0, _gcmIvLength);
  final ct = packed.sublist(_gcmIvLength);
  final cipher = GCMBlockCipher(AESEngine())
    ..init(
      false,
      AEADParameters(KeyParameter(key), _gcmTagLength * 8, iv, Uint8List(0)),
    );
  final tmp = Uint8List(ct.length);
  var n = cipher.processBytes(ct, 0, ct.length, tmp, 0);
  n += cipher.doFinal(tmp, n);
  return tmp.sublist(0, n);
}

Uint8List _randomBytes(int length) {
  final random = Random.secure();
  final out = Uint8List(length);
  for (var i = 0; i < length; i++) {
    out[i] = random.nextInt(256);
  }
  return out;
}
