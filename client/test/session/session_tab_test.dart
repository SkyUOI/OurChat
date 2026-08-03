import 'package:flutter_test/flutter_test.dart';

/// Pure unit tests for the inline file-drop handler logic.
/// We can't easily mock desktop_drop's DropDoneDetails, so we test the
/// equivalent processing functions extracted from _SessionTabState.
class _CacheFileHelper {
  /// The exact same content-type detection logic used in _cacheFileForUpload.
  static bool isImageContentType(String contentType) {
    return contentType.startsWith('image/');
  }

  /// Build the markdown insertion text — mirrors _cacheFileForUpload.
  static String buildInsertion({
    required String currentText,
    required String filename,
    required String path,
    required bool isImage,
  }) {
    final breakLine = currentText.isEmpty || currentText.endsWith('\n')
        ? ''
        : '\n';
    if (isImage) {
      return '$currentText$breakLine![$filename]($path)';
    }
    return '$currentText$breakLine[$filename]($path)';
  }
}

void main() {
  group('isImageContentType', () {
    test('image/png is image', () {
      expect(_CacheFileHelper.isImageContentType('image/png'), true);
    });

    test('image/jpeg is image', () {
      expect(_CacheFileHelper.isImageContentType('image/jpeg'), true);
    });

    test('image/gif is image', () {
      expect(_CacheFileHelper.isImageContentType('image/gif'), true);
    });

    test('video/mp4 is not image', () {
      expect(_CacheFileHelper.isImageContentType('video/mp4'), false);
    });

    test('application/pdf is not image', () {
      expect(_CacheFileHelper.isImageContentType('application/pdf'), false);
    });

    test('application/octet-stream is not image', () {
      expect(
        _CacheFileHelper.isImageContentType('application/octet-stream'),
        false,
      );
    });
  });

  group('buildInsertion', () {
    test('image prepends ! and uses image syntax', () {
      final result = _CacheFileHelper.buildInsertion(
        currentText: '',
        filename: 'photo.png',
        path: '/tmp/photo.png',
        isImage: true,
      );
      expect(result, '![photo.png](/tmp/photo.png)');
    });

    test('non-image file uses link syntax (no !)', () {
      final result = _CacheFileHelper.buildInsertion(
        currentText: '',
        filename: 'doc.pdf',
        path: '/tmp/doc.pdf',
        isImage: false,
      );
      expect(result, '[doc.pdf](/tmp/doc.pdf)');
    });

    test('appends to existing text with newline', () {
      final result = _CacheFileHelper.buildInsertion(
        currentText: 'hello',
        filename: 'f.txt',
        path: '/tmp/f.txt',
        isImage: false,
      );
      expect(result, 'hello\n[f.txt](/tmp/f.txt)');
    });

    test('no double newline when text ends with newline', () {
      final result = _CacheFileHelper.buildInsertion(
        currentText: 'hello\n',
        filename: 'f.txt',
        path: '/tmp/f.txt',
        isImage: false,
      );
      expect(result, 'hello\n[f.txt](/tmp/f.txt)');
    });

    test('image appends to existing text', () {
      final result = _CacheFileHelper.buildInsertion(
        currentText: 'look:',
        filename: 'img.jpg',
        path: '/tmp/img.jpg',
        isImage: true,
      );
      expect(result, 'look:\n![img.jpg](/tmp/img.jpg)');
    });
  });
}
