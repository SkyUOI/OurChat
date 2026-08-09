import 'dart:convert';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:typed_data';

import 'package:web/web.dart' as web;

/// Generate an RSA-2048 keypair using the browser's native WebCrypto
/// (`crypto.subtle.generateKey`, typically ~100ms) instead of the slow pure-
/// Dart pointycastle keygen, which can block the UI thread for ~20s on web.
///
/// Keys are returned as PKCS#1 DER — the same format the rest of the app
/// (pointycastle `decodeRsaPublicKey` / `decodeRsaPrivateKey`) uses.
Future<({Uint8List publicKey, Uint8List privateKey})>
generateRsaKeyPairAsync() async {
  final subtle = web.window.crypto.subtle;

  final algorithm = JSObject()
    ..['name'] = 'RSA-OAEP'.toJS
    ..['modulusLength'] = 2048.toJS
    ..['publicExponent'] = Uint8List.fromList([1, 0, 1]).toJS
    ..['hash'] = 'SHA-256'.toJS;

  final keyPairAny = await subtle
      .generateKey(algorithm, true, ['encrypt'.toJS, 'decrypt'.toJS].toJS)
      .toDart;
  final keyPair = keyPairAny as JSObject;
  final publicKey = keyPair['publicKey'] as web.CryptoKey;
  final privateKey = keyPair['privateKey'] as web.CryptoKey;

  final publicJwk =
      (await subtle.exportKey('jwk', publicKey).toDart) as web.JsonWebKey;
  final privateJwk =
      (await subtle.exportKey('jwk', privateKey).toDart) as web.JsonWebKey;

  return (
    publicKey: _encodePublicPkcs1(publicJwk),
    privateKey: _encodePrivatePkcs1(privateJwk),
  );
}

// ── JWK → PKCS#1 DER ──────────────────────────────────────────────────────

BigInt _jwkBigInt(String? b64url) {
  final s = b64url ?? '';
  final padded = s
      .padRight(((s.length + 3) ~/ 4) * 4, '=')
      .replaceAll('-', '+')
      .replaceAll('_', '/');
  final bytes = base64.decode(padded);
  return BigInt.parse(
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join(),
    radix: 16,
  );
}

Uint8List _encodeInteger(BigInt value) {
  if (value == BigInt.zero) {
    return Uint8List.fromList([0x02, 0x01, 0x00]);
  }
  var hex = value.toRadixString(16);
  if (hex.length % 2 != 0) hex = '0$hex';
  final bytes = Uint8List(hex.length ~/ 2);
  for (var i = 0; i < bytes.length; i++) {
    bytes[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
  }
  final withSign = (bytes[0] & 0x80) != 0
      ? Uint8List.fromList([0x00, ...bytes])
      : bytes;
  return Uint8List.fromList([
    0x02,
    ..._encodeLength(withSign.length),
    ...withSign,
  ]);
}

Uint8List _encodeLength(int length) {
  if (length < 128) return Uint8List.fromList([length]);
  final bytes = <int>[];
  var l = length;
  while (l > 0) {
    bytes.insert(0, l & 0xFF);
    l >>= 8;
  }
  return Uint8List.fromList([0x80 | bytes.length, ...bytes]);
}

Uint8List _encodeSequence(Uint8List contents) =>
    Uint8List.fromList([0x30, ..._encodeLength(contents.length), ...contents]);

Uint8List _encodePublicPkcs1(web.JsonWebKey jwk) {
  final n = _encodeInteger(_jwkBigInt(jwk.n));
  final e = _encodeInteger(_jwkBigInt(jwk.e));
  return _encodeSequence(Uint8List.fromList([...n, ...e]));
}

Uint8List _encodePrivatePkcs1(web.JsonWebKey jwk) {
  final version = _encodeInteger(BigInt.zero);
  final n = _encodeInteger(_jwkBigInt(jwk.n));
  final e = _encodeInteger(_jwkBigInt(jwk.e));
  final d = _encodeInteger(_jwkBigInt(jwk.d));
  final p = _encodeInteger(_jwkBigInt(jwk.p));
  final q = _encodeInteger(_jwkBigInt(jwk.q));
  final dp = _encodeInteger(_jwkBigInt(jwk.dp));
  final dq = _encodeInteger(_jwkBigInt(jwk.dq));
  final qi = _encodeInteger(_jwkBigInt(jwk.qi));
  return _encodeSequence(
    Uint8List.fromList([
      ...version,
      ...n,
      ...e,
      ...d,
      ...p,
      ...q,
      ...dp,
      ...dq,
      ...qi,
    ]),
  );
}
