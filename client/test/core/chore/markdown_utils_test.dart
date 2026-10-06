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

  group('isHttpUrl', () {
    test('accepts http and https', () {
      expect(isHttpUrl('http://example.com/a.png'), isTrue);
      expect(isHttpUrl('https://example.com/a.png'), isTrue);
    });

    test('rejects other schemes and plain paths', () {
      expect(isHttpUrl('io://0'), isFalse);
      expect(isHttpUrl('in://https,example.com/a.png'), isFalse);
      expect(isHttpUrl('/tmp/file.png'), isFalse);
      expect(isHttpUrl(''), isFalse);
    });
  });

  group('encodeExternalImageUrl / decodeExternalImageUrl', () {
    test('encodes the scheme separator as a comma', () {
      expect(
        encodeExternalImageUrl('https://example.com/a.png'),
        'in://https,example.com/a.png',
      );
      expect(
        encodeExternalImageUrl('http://example.com/a.png'),
        'in://http,example.com/a.png',
      );
    });

    test('round-trips urls that themselves contain commas', () {
      const url = 'https://example.com/a,b/c,d.png?x=1,2';
      expect(decodeExternalImageUrl(encodeExternalImageUrl(url)), url);
    });

    test('decode mirrors the imageBuilder in:// parsing', () {
      // The renderer builds the url back from the comma-separated content:
      // `path[0] + "://" + path.sublist(1).join(",")`.
      const encoded = 'in://https,example.com/a.png';
      final content = encoded.split('://')[1];
      final path = content.split(',');
      expect(
        '${path[0]}://${path.sublist(1).join(',')}',
        decodeExternalImageUrl(content),
      );
    });
  });

  group('extractMarkdownHttpImageUrls', () {
    test('finds http and https image urls', () {
      const input = 'a ![x](https://e.com/a.png) b ![y](http://e.net/b.jpg)';
      expect(extractMarkdownHttpImageUrls(input), [
        'https://e.com/a.png',
        'http://e.net/b.jpg',
      ]);
    });

    test('ignores non-http images and plain links', () {
      const input =
          '![a](io://0) ![b](in://https,e.com/x.png) [c](https://e.com)';
      expect(extractMarkdownHttpImageUrls(input), isEmpty);
    });

    test('matches images with a title', () {
      const input = '![alt](https://e.com/a.png "the title")';
      expect(extractMarkdownHttpImageUrls(input), ['https://e.com/a.png']);
    });

    test('deduplicates repeated urls', () {
      const input = '![a](https://e.com/a.png) ![b](https://e.com/a.png)';
      expect(extractMarkdownHttpImageUrls(input), ['https://e.com/a.png']);
    });

    test('empty input returns empty', () {
      expect(extractMarkdownHttpImageUrls(''), isEmpty);
    });
  });

  group('rewriteMarkdownHttpImagesToIn', () {
    test('rewrites http image urls preserving alt and title', () {
      const input = '![cat](https://e.com/cat.png "a cat")';
      expect(
        rewriteMarkdownHttpImagesToIn(input),
        '![cat](in://https,e.com/cat.png "a cat")',
      );
    });

    test('leaves non-http images and links untouched', () {
      const input = '![a](io://0) [b](https://e.com) text';
      expect(rewriteMarkdownHttpImagesToIn(input), input);
    });

    test('leaves surrounding markdown untouched', () {
      const input = '# Title\n\n![a](https://e.com/a.png)\n\n- item\n';
      expect(
        rewriteMarkdownHttpImagesToIn(input),
        '# Title\n\n![a](in://https,e.com/a.png)\n\n- item\n',
      );
    });
  });

  group('rewriteMarkdownHttpImageUrls', () {
    test('rewrites only the matching url (io:// upload style)', () {
      const input = '![a](https://e.com/a.png) ![b](https://e.com/b.png)';
      final result = rewriteMarkdownHttpImageUrls(input, (url) {
        if (url == 'https://e.com/a.png') return 'IO://0';
        return url;
      });
      expect(result, '![a](IO://0) ![b](https://e.com/b.png)');
    });

    test('empty input returns empty', () {
      expect(rewriteMarkdownHttpImageUrls('', (url) => 'X'), '');
    });
  });
}
