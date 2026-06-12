import 'dart:math';
import 'dart:typed_data';
import 'package:pointycastle/export.dart';

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

  return _encodeSequence(Uint8List.fromList([
    ...version,
    ...modulus,
    ...publicExponent,
    ...privateExponent,
    ...p,
    ...q,
    ...dP,
    ...dQ,
    ...qInv,
  ]));
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
    List.generate(bytes.length ~/ 2, (i) => int.parse(bytes.substring(i * 2, i * 2 + 2), radix: 16)),
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
