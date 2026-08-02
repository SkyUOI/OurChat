import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore/markdown_utils.dart';

/// Tests for the pure (non-l10n) functions in `markdown_utils.dart`:
/// `MarkdownToText.containsImage` and `replaceMarkdownImageUrls`.
///
/// `MarkdownToText.convert` requires a live `AppLocalizations` (it is abstract
/// and can only be obtained through a MaterialApp), so it is not unit-tested
/// here.
void main() {
  group('MarkdownToText.containsImage', () {
    test('plain text returns false', () {
      expect(MarkdownToText.containsImage('hello world'), false);
    });

    test('image returns true', () {
      expect(MarkdownToText.containsImage('see ![img](file.png)'), true);
    });

    test('link without ! returns false', () {
      expect(MarkdownToText.containsImage('[file](file.png)'), false);
    });

    test('empty string returns false', () {
      expect(MarkdownToText.containsImage(''), false);
    });

    test('detects image in mixed content', () {
      expect(
        MarkdownToText.containsImage('text **bold** ![img](x.png) more'),
        true,
      );
    });
  });

  group('replaceMarkdownImageUrls', () {
    test('replaces an image src', () {
      final result = replaceMarkdownImageUrls('![a](/old.png)', (url) {
        if (url == '/old.png') return '/new.png';
        return url;
      });
      expect(result, contains('![a](/new.png)'));
      expect(result, isNot(contains('/old.png')));
    });

    test('does NOT touch link hrefs (images only)', () {
      final result = replaceMarkdownImageUrls('[file](/old.dat)', (url) {
        fail('replaceUrl must not be called for non-image links');
      });
      expect(result, contains('/old.dat'));
    });

    test('leaves unrelated image URLs untouched', () {
      const input = '[a](/a.dat) ![b](/b.png) [c](/c.dat)';
      final result = replaceMarkdownImageUrls(input, (url) {
        if (url == '/b.png') return 'IO://1';
        return url;
      });
      expect(result, contains('[a](/a.dat)'));
      expect(result, contains('![b](IO://1)'));
      expect(result, contains('[c](/c.dat)'));
    });

    test('replaces multiple image occurrences', () {
      const input = '![a](/a.png) ![b](/a.png)';
      final result = replaceMarkdownImageUrls(input, (url) {
        if (url == '/a.png') return 'IO://0';
        return url;
      });
      expect('IO://0'.allMatches(result).length, 2);
    });

    test('replaceUrl receives the original src', () {
      final captured = <String>[];
      replaceMarkdownImageUrls('![x](/p.png) ![y](/q.png)', (url) {
        captured.add(url);
        return url;
      });
      expect(captured, ['/p.png', '/q.png']);
    });

    test('empty input returns empty', () {
      expect(replaceMarkdownImageUrls('', (url) => 'X'), '');
    });
  });
}
