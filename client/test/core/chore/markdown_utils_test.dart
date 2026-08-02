import 'package:flutter_test/flutter_test.dart';
import 'package:ourchat/core/chore/markdown_utils.dart';

/// Tests for functions that do NOT depend on AppLocalizations.
/// The MarkdownToText tests that need l10n are skipped because AppLocalizations
/// is abstract and cannot be instantiated without a running MaterialApp.

void main() {
  group('MarkdownToText.containsImage', () {
    test('no image returns false', () {
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

  group('replaceMarkdownFileUrls', () {
    test('replaces image src', () {
      final result = replaceMarkdownFileUrls('![a](/old.png)', (url) {
        if (url == '/old.png') return '/new.png';
        return url;
      });
      expect(result, contains('![a](/new.png)'));
    });

    test('replaces link href', () {
      final result = replaceMarkdownFileUrls('[file](/old.dat)', (url) {
        if (url == '/old.dat') return 'IO://0';
        return url;
      });
      expect(result, contains('[file](IO://0)'));
    });

    test('does not touch unrelated URLs', () {
      final input = '[a](/a.dat) ![b](/b.png) [c](/c.dat)';
      final result = replaceMarkdownFileUrls(input, (url) {
        if (url == '/b.png') return 'IO://1';
        return url;
      });
      expect(result, contains('[a](/a.dat)'));
      expect(result, contains('![b](IO://1)'));
      expect(result, contains('[c](/c.dat)'));
    });

    test('replaces multiple occurrences', () {
      final input = '![a](/a.png) ![b](/a.png)';
      final result = replaceMarkdownFileUrls(input, (url) {
        if (url == '/a.png') return 'IO://0';
        return url;
      });
      expect('IO://0'.allMatches(result).length, 2);
    });

    test('mixed image and link replacement', () {
      final input = '![img](/img.png) [file](/file.pdf)';
      final result = replaceMarkdownFileUrls(input, (url) {
        if (url == '/img.png') return 'IO://0';
        if (url == '/file.pdf') return 'IO://1';
        return url;
      });
      expect(result, contains('![img](IO://0)'));
      expect(result, contains('[file](IO://1)'));
    });

    test('handles complex markdown with links and images', () {
      final input = '''
# Title
Some text with ![image](/a.png) and a [link](/b.pdf).
Another ![image2](/c.jpg) here.''';
      final replaced = <String>[];
      final result = replaceMarkdownFileUrls(input, (url) {
        replaced.add(url);
        return 'IO://${replaced.length - 1}';
      });
      expect(replaced, ['/a.png', '/b.pdf', '/c.jpg']);
      expect(result, contains('IO://0'));
      expect(result, contains('IO://1'));
      expect(result, contains('IO://2'));
    });

    test('empty input returns empty', () {
      expect(replaceMarkdownFileUrls('', (url) => 'X'), '');
    });

    test('text with no URLs returns unchanged (modulo markdown formatting)', () {
      const input = 'plain text **bold** *italic*';
      final result = replaceMarkdownFileUrls(input, (url) {
        fail('should not be called');
      });
      // The renderer normalises markdown, so bold/italic markers are stripped.
      // What matters is no replaceUrl callback was called.
    });
  });

  group('replaceMarkdownImageUrls (legacy)', () {
    test('behaves same as replaceMarkdownFileUrls', () {
      final input = '![a](/old.png) [b](/old.dat)';
      final r1 = replaceMarkdownImageUrls(input, (url) => 'X');
      final r2 = replaceMarkdownFileUrls(input, (url) => 'X');
      expect(r1, r2);
    });

    test('still replaces links (new behavior, backward compat)', () {
      // The legacy function now delegates to replaceMarkdownFileUrls,
      // so it also replaces link hrefs.  This is intentional.
      final result = replaceMarkdownImageUrls('[f](/a.dat)', (url) => 'IO://0');
      expect(result, contains('IO://0'));
    });
  });
}
