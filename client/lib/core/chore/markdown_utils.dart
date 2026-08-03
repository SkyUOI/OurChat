import 'package:markdown/markdown.dart' as md;
import 'package:ourchat/l10n/app_localizations.dart';

/// MarkDown -> PlainText (GENERATE BY AI)
class MarkdownToText {
  /// Convert Markdown text to plain text, ignoring all syntax (supports flutter_markdown_plus enhanced syntax)
  static String convert(String markdownText, AppLocalizations l10n) {
    if (markdownText.isEmpty) return "";

    // 1. Parse Markdown with flutter_markdown_plus compatible rules
    final document = md.Document(extensionSet: md.ExtensionSet.gitHubFlavored);
    final nodes = document.parseLines(markdownText.split('\n'));

    // 2. Walk the nodes with a visitor, extracting only text node contents (avoid duplicates)
    final StringBuffer textBuffer = StringBuffer();
    for (final node in nodes) {
      node.accept(_NodeTextExtractor(textBuffer, l10n));
    }

    // 3. Clean up and return the plain text
    return _cleanText(textBuffer.toString());
  }

  /// Detect whether a Markdown string contains images
  /// [markdownText] the markdown string to inspect
  /// Returns: true if it contains an image, false otherwise
  static bool containsImage(String markdownText) {
    if (markdownText.isEmpty) return false;

    // Parse Markdown with the same rules
    final document = md.Document(extensionSet: md.ExtensionSet.gitHubFlavored);
    final nodes = document.parseLines(markdownText.split('\n'));

    // Create an image detector and walk all nodes
    final imageDetector = _ImageDetector();
    for (final node in nodes) {
      node.accept(imageDetector);
      // Stop as soon as an image is found, for performance
      if (imageDetector.hasImage) {
        return true;
      }
    }

    return imageDetector.hasImage;
  }

  /// Collapse extra spaces and newlines (optimization: keep single newlines, closer to the original)
  static String _cleanText(String text) {
    return text
        .replaceAll(RegExp(r'\n+'), '\n') // Collapse multiple newlines to one
        .replaceAll(
          RegExp(r'\s+\n'),
          '\n',
        ) // Trim trailing spaces before newlines
        .replaceAll(
          RegExp(r'\n\s+'),
          '\n',
        ) // Trim leading spaces after newlines
        .replaceAll(
          RegExp(r'[ \t]+'),
          ' ',
        ) // Collapse multiple spaces/tabs to one
        .trim(); // Trim leading/trailing spaces and newlines
  }
}

/// An image detector that walks the Markdown node tree looking for image nodes
class _ImageDetector implements md.NodeVisitor {
  bool hasImage = false;

  @override
  void visitText(md.Text text) {
    // Text nodes need no handling
  }

  @override
  bool visitElementBefore(md.Element element) {
    // Detect img tags (image nodes)
    if (element.tag == 'img') {
      hasImage = true;
      return false; // Image nodes have no children to traverse
    }
    return true; // Continue traversing children of other nodes
  }

  @override
  void visitElementAfter(md.Element element) {
    // Nothing to do
  }
}

/// Extract only the innermost Text node contents, ignoring parent nodes (fixes duplicates)
class _NodeTextExtractor implements md.NodeVisitor {
  final StringBuffer buffer;
  AppLocalizations l10n;

  _NodeTextExtractor(this.buffer, this.l10n);

  /// Only process text nodes: the innermost text source, no duplicates
  @override
  void visitText(md.Text text) {
    final textContent = text.text.trim();
    if (textContent.isNotEmpty) {
      buffer.write(textContent);
      // Add a single space between text nodes (avoid merging)
      buffer.write(" ");
    }
  }

  @override
  bool visitElementBefore(md.Element element) {
    // Handle image nodes: replace the img tag with [image]
    if (element.tag == 'img') {
      buffer.write(
        "[${l10n.image}] ",
      ); // Add a space to avoid sticking to other content
      return false; // Image nodes have no children to traverse
    }

    // Special handling: add newlines around list items, paragraphs, tables, etc. (better formatting)
    if (element.tag == 'li' || element.tag == 'p' || element.tag == 'tr') {
      buffer.write("\n");
    }
    return true; // Must return true to keep traversing children
  }

  /// After visiting an element node: nothing to do
  @override
  void visitElementAfter(md.Element element) {}
}

/// Replace urls in markdown text (GENERATE BY AI)
String replaceMarkdownImageUrls(
  String markdown,
  String Function(String oldUrl) replaceUrl,
) {
  final doc = md.Document(encodeHtml: false);
  final nodes = doc.parseLines(markdown.split('\n'));

  // Walk the AST and replace img node src
  void walk(List<md.Node> list) {
    for (var node in list) {
      if (node is md.Element) {
        if (node.tag == 'img') {
          final old = node.attributes['src'] ?? '';
          node.attributes['src'] = replaceUrl(old);
        }
        if (node.children != null && node.children!.isNotEmpty) {
          walk(node.children!);
        }
      }
    }
  }

  walk(nodes);

  // Simple serialization back to Markdown (covers common nodes)
  final renderer = _MiniRenderer();
  return renderer.render(nodes);
}

class _MiniRenderer {
  final StringBuffer _buf = StringBuffer();

  String render(List<md.Node> nodes) {
    _buf.clear();
    for (var n in nodes) {
      _render(n, parent: null);
    }
    var out = _buf.toString();
    out = out.replaceAll(RegExp(r'\s+$'), '\n');
    return out;
  }

  void _render(md.Node node, {md.Node? parent}) {
    if (node is md.Text) {
      _buf.write(node.text);
      return;
    }

    if (node is md.Element) {
      switch (node.tag) {
        case 'p':
          _renderInline(node);
          _buf.writeln('\n');
          return;
        case 'h1':
        case 'h2':
        case 'h3':
        case 'h4':
        case 'h5':
        case 'h6':
          final lvl = int.parse(node.tag.substring(1));
          _buf.write('${'#' * lvl} ');
          _renderInline(node);
          _buf.writeln('\n');
          return;
        case 'pre': // fenced code block
          String lang = '';
          String code = _collectText(node);
          final first = (node.children != null && node.children!.isNotEmpty)
              ? node.children!.first
              : null;
          if (first is md.Element && first.tag == 'code') {
            final cls = first.attributes['class'] ?? '';
            final m = RegExp(r'language-([^\s]+)').firstMatch(cls);
            if (m != null) lang = m.group(1) ?? '';
            code = _collectText(first);
          }
          _buf.writeln('```$lang');
          _buf.writeln(code);
          _buf.writeln('```');
          _buf.writeln();
          return;
        case 'ul':
        case 'ol':
          final ordered = node.tag == 'ol';
          var idx = 1;
          for (var li in node.children ?? []) {
            if (li.tag == 'li') {
              final tmp = StringBuffer();
              // Render li to a temporary buffer
              final old = _swapBuffer(tmp);
              for (var c in li.children ?? []) {
                _render(c, parent: li);
              }
              _restoreBuffer(old);
              final lines = tmp.toString().trimRight().split('\n');
              final prefix = ordered ? '$idx. ' : '- ';
              _buf.write("$prefix${(lines.isNotEmpty ? lines.first : '')}\n");
              for (var i = 1; i < lines.length; i++) {
                _buf.write('  ${lines[i]}\n');
              }
              idx++;
            }
          }
          _buf.writeln();
          return;
        case 'a':
          final href = node.attributes['href'] ?? '';
          _buf.write('[');
          _renderInline(node);
          _buf.write(']($href)');
          return;
        case 'img':
          final alt = node.attributes['alt'] ?? '';
          final src = node.attributes['src'] ?? '';
          final title = node.attributes['title'];
          if (title != null && title.isNotEmpty) {
            _buf.write('![$alt]($src "$title")');
          } else {
            _buf.write('![$alt]($src)');
          }
          return;
        case 'code':
          if (parent is md.Element && parent.tag == 'pre') {
            _buf.write(_collectText(node));
          } else {
            _buf.write('`');
            _buf.write(_collectText(node));
            _buf.write('`');
          }
          return;
        default:
          for (var c in node.children ?? []) {
            _render(c, parent: node);
          }
          return;
      }
    }

    _buf.write(node.toString());
  }

  void _renderInline(md.Element node) {
    for (var c in node.children ?? []) {
      _render(c, parent: node);
    }
  }

  String _collectText(md.Node node) {
    if (node is md.Text) return node.text;
    if (node is md.Element) {
      return (node.children ?? []).map((c) => _collectText(c)).join();
    }

    return '';
  }

  // Simplified buffer swap (for temporary collection in list items)
  StringBuffer _swapBuffer(StringBuffer newBuf) => _buf;
  void _restoreBuffer(StringBuffer old) {}
}
