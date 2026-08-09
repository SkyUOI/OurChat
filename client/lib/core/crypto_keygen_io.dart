import 'dart:typed_data';

import 'crypto.dart';

/// On desktop/mobile the pointycastle RSA-2048 keygen is fast enough to run on
/// the UI thread, so just wrap the synchronous implementation.
Future<({Uint8List publicKey, Uint8List privateKey})>
generateRsaKeyPairAsync() async => generateRsaKeyPair();
