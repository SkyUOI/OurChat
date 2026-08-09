import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ourchat/core/crypto.dart';

/// Verifies the web keypair generator (native WebCrypto) produces PKCS#1 DER
/// keys that the existing pointycastle E2EE code can decode and use, and that
/// generation is fast enough to not freeze the UI.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('web: WebCrypto keygen is fast and pointycastle-compatible', (
    _,
  ) async {
    final sw = Stopwatch()..start();
    final pair = await generateRsaKeyPairAsync();
    sw.stop();
    IntegrationTestWidgetsFlutterBinding.instance.reportData = {
      'weccrypto_keygen_ms': sw.elapsedMilliseconds,
    };

    // Decode with the same pointycastle parsers the app uses.
    final pub = decodeRsaPublicKey(pair.publicKey);
    final priv = decodeRsaPrivateKey(pair.privateKey);
    expect(pub.modulus, isNotNull);
    expect(priv.modulus, isNotNull);
    expect(
      pub.modulus,
      priv.modulus,
      reason: 'public and private key must share the modulus',
    );

    // Round-trip: encrypt with the public key, decrypt with the private key.
    const secret = 'web-crypto-roundtrip';
    final ct = rsaEncrypt(
      pair.publicKey,
      Uint8List.fromList(utf8.encode(secret)),
    );
    expect(utf8.decode(rsaDecrypt(pair.privateKey, ct)), secret);

    expect(
      sw.elapsed,
      lessThan(const Duration(seconds: 3)),
      reason:
          'web keygen must not freeze the UI '
          '(took ${sw.elapsedMilliseconds}ms)',
    );
  });
}
