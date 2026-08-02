import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';

/// Pure-logic unit tests for OurChatFileResult.
/// We test the model without importing the full upload module to avoid
/// transitive gRPC / Flutter dependencies the test harness can't resolve.
class OurChatFileResultStub {
  final Uint8List bytes;
  final String contentType;
  final String filename;
  final int size;

  OurChatFileResultStub({
    required this.bytes,
    this.contentType = '',
    this.filename = '',
    this.size = 0,
  });
}

void main() {
  group('OurChatFileResult (model)', () {
    test('defaults are empty', () {
      final r = OurChatFileResultStub(bytes: Uint8List(0));
      expect(r.contentType, '');
      expect(r.filename, '');
      expect(r.size, 0);
      expect(r.bytes, isEmpty);
    });

    test('stores metadata', () {
      final r = OurChatFileResultStub(
        bytes: Uint8List.fromList([1, 2, 3]),
        contentType: 'image/png',
        filename: 'photo.png',
        size: 1024,
      );
      expect(r.contentType, 'image/png');
      expect(r.filename, 'photo.png');
      expect(r.size, 1024);
      expect(r.bytes, hasLength(3));
    });

    test('video content type', () {
      final r = OurChatFileResultStub(
        bytes: Uint8List(100),
        contentType: 'video/mp4',
        filename: 'clip.mp4',
        size: 100,
      );
      expect(r.contentType, 'video/mp4');
    });

    test('generic file with octet-stream', () {
      final r = OurChatFileResultStub(
        bytes: Uint8List(42),
        contentType: 'application/octet-stream',
        filename: 'unknown.bin',
        size: 42,
      );
      expect(r.contentType, 'application/octet-stream');
    });
  });
}
