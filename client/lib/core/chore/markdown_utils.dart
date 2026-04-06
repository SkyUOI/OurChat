import 'package:markdown/markdown.dart' as md;
import 'package:ourchat/l10n/app_localizations.dart';

/// MarkDown -> PlainText (GENERATE BY AI)
class MarkdownToText {
  /// 将 Markdown 文本转为纯文本，忽略所有语法（支持 flutter_markdown_plus 增强语法）
  static String convert(String markdownText, AppLocalizations l10n) {
    if (markdownText.isEmpty) return "";

    // 1. 用 flutter_markdown_plus 兼容的规则解析 Markdown
    final document = md.Document(extensionSet: md.ExtensionSet.gitHubFlavored);
    final nodes = document.parseLines(markdownText.split('\n'));

    // 2. 用访问器遍历节点，只提取文本节点内容（避免重复）
    final StringBuffer textBuffer = StringBuffer();
    for (final node in nodes) {
      node.accept(_NodeTextExtractor(textBuffer, l10n));
    }

    // 3. 清理并返回纯文本
    return _cleanText(textBuffer.toString());
  }

  /// 新增：检测 Markdown 字符串中是否包含图片
  /// [markdownText] 需要检测的 markdown 字符串
  /// 返回值：true 包含图片，false 不包含图片
  static bool containsImage(String markdownText) {
    if (markdownText.isEmpty) return false;

    // 使用相同的解析规则解析 Markdown
    final document = md.Document(extensionSet: md.ExtensionSet.gitHubFlavored);
    final nodes = document.parseLines(markdownText.split('\n'));

    // 创建图片检测器并遍历所有节点
    final imageDetector = _ImageDetector();
    for (final node in nodes) {
      node.accept(imageDetector);
      // 一旦检测到图片，立即返回 true，提升性能
      if (imageDetector.hasImage) {
        return true;
      }
    }

    return imageDetector.hasImage;
  }

  /// 清理多余空格和换行（优化：保留单个换行，更贴近原文结构）
  static String _cleanText(String text) {
    return text
        .replaceAll(RegExp(r'\n+'), '\n') // 多个换行 → 单个换行
        .replaceAll(RegExp(r'\s+\n'), '\n') // 换行前的多余空格 → 仅保留换行
        .replaceAll(RegExp(r'\n\s+'), '\n') // 换行后的多余空格 → 仅保留换行
        .replaceAll(RegExp(r'[ \t]+'), ' ') // 多个空格/制表符 → 单个空格
        .trim(); // 去除首尾空格和换行
  }
}

/// 图片检测器：遍历 Markdown 节点树，检测是否包含图片节点
class _ImageDetector implements md.NodeVisitor {
  bool hasImage = false;

  @override
  void visitText(md.Text text) {
    // 文本节点无需处理
  }

  @override
  bool visitElementBefore(md.Element element) {
    // 检测 img 标签（图片节点）
    if (element.tag == 'img') {
      hasImage = true;
      return false; // 图片节点无子节点，无需继续遍历
    }
    return true; // 继续遍历其他节点的子节点
  }

  @override
  void visitElementAfter(md.Element element) {
    // 无需处理
  }
}

/// 修复重复文本：仅提取最底层文本节点（Text）的内容，忽略父节点
class _NodeTextExtractor implements md.NodeVisitor {
  final StringBuffer buffer;
  AppLocalizations l10n;

  _NodeTextExtractor(this.buffer, this.l10n);

  /// 只处理文本节点：这是最底层的文本来源，不会重复
  @override
  void visitText(md.Text text) {
    final textContent = text.text.trim();
    if (textContent.isNotEmpty) {
      buffer.write(textContent);
      // 文本节点之间添加单个空格（避免连在一起）
      buffer.write(" ");
    }
  }

  @override
  bool visitElementBefore(md.Element element) {
    // 处理图片节点：img 标签替换为 [图片]
    if (element.tag == 'img') {
      buffer.write("[${l10n.image}] "); // 添加空格避免和其他内容粘连
      return false; // 图片节点无子节点，无需继续遍历
    }

    // 特殊处理：列表项、段落、表格等节点，添加换行分隔（优化格式）
    if (element.tag == 'li' || element.tag == 'p' || element.tag == 'tr') {
      buffer.write("\n");
    }
    return true; // 必须返回 true，才会继续遍历子节点
  }

  /// 访问元素节点之后：无需处理
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

  // 遍历 AST 并替换 img 节点的 src
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

  // 简单序列化回 Markdown（覆盖常见节点）
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
              // 渲染 li 到临时 buffer
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

  // 简化版 buffer swap（用于列表项临时收集）
  StringBuffer _swapBuffer(StringBuffer newBuf) => _buf;
  void _restoreBuffer(StringBuffer old) {}
}
