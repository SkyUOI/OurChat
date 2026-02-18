import 'dart:convert';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_cache_manager/flutter_cache_manager.dart';
import 'package:hashlib/hashlib.dart';
import 'package:image_compression/image_compression.dart';
import 'package:markdown/markdown.dart' as md;
import 'package:grpc/grpc.dart';
import 'package:ourchat/core/log.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:http/http.dart' as http;
import 'package:ourchat/service/ourchat/download/v1/download.pb.dart';
import 'dart:async';

import 'package:ourchat/service/ourchat/upload/v1/upload.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

class OurChatTime {
  /*
  该类用于grpc的timestamp与datetime两种时间类型之间的转换
  为了避免数据库的精度问题，转换所得的datetime并不准确
   */
  Timestamp? inputTimestamp;
  DateTime? inputDatetime;
  late Timestamp timestamp;
  late DateTime datetime;

  OurChatTime.fromTimestamp(Timestamp ts) {
    timestamp = ts;
    toDatetime();
  }

  OurChatTime.fromDatetime(DateTime dt) {
    datetime = dt;
    toTimestamp();
  }

  void toTimestamp() {
    Int64 seconds = Int64.parseInt(
        (datetime.microsecondsSinceEpoch / 1000000).round().toString());
    // print(datetime.microsecondsSinceEpoch);
    // print("=>timestamp$seconds,$nanos");
    timestamp = Timestamp(seconds: seconds);
  }

  void toDatetime() {
    var seconds = timestamp.seconds;
    // print(timestamp);
    // print("=>datetime${seconds.toInt() * 1000000 + nanos}");
    datetime = DateTime.fromMicrosecondsSinceEpoch(seconds.toInt() * 1000000);
  }

  @override
  bool operator ==(Object other) {
    if (other is OurChatTime) {
      return datetime.difference(other.datetime).inMicroseconds == 0;
    }
    return false;
  }

  @override
  int get hashCode => timestamp.hashCode;
}

void showResultMessage(
  OurChatAppState ourchatAppState,
  int code,
  String? errorMessage, {
  dynamic okStatus,
  dynamic cancelledStatus,
  dynamic unknownStatus,
  dynamic invalidArgumentStatus,
  dynamic deadlineExceededStatus,
  dynamic notFoundStatus,
  dynamic alreadyExistsStatus,
  dynamic permissionDeniedStatus,
  dynamic resourceExhaustedStatus,
  dynamic failedPreconditionStatus,
  dynamic abortedStatus,
  dynamic outOfRangeStatus,
  dynamic unimplementedStatus,
  dynamic internalStatus,
  dynamic unavailableStatus,
  dynamic dataLossStatus,
  dynamic unauthenticatedStatus,
}) {
  var l10n = ourchatAppState.l10n;
  dynamic message = l10n.unknownError;
  switch (code) {
    case okStatusCode:
      message = l10n.succeeded;
      if (okStatus != null) {
        message = okStatus;
      }
      break;
    case cancelledStatusCode:
      if (cancelledStatus != null) {
        message = cancelledStatus;
      }
      break;
    case unknownStatusCode:
      if (unknownStatus != null) {
        message = unknownStatus;
      }
      break;
    case invalidArgumentStatusCode:
      if (invalidArgumentStatus != null) {
        message = invalidArgumentStatus;
      }
      break;
    case deadlineExceededStatusCode:
      if (deadlineExceededStatus != null) {
        message = deadlineExceededStatus;
      }
      break;
    case notFoundStatusCode:
      if (notFoundStatus != null) {
        message = notFoundStatus;
      }
      break;
    case alreadyExistsStatusCode:
      if (alreadyExistsStatus != null) {
        message = alreadyExistsStatus;
      }
      break;
    case permissionDeniedStatusCode:
      if (permissionDeniedStatus != null) {
        message = permissionDeniedStatus;
      }
      break;
    case resourceExhaustedStatusCode:
      if (resourceExhaustedStatus != null) {
        message = resourceExhaustedStatus;
      }
      break;
    case failedPreconditionStatusCode:
      if (failedPreconditionStatus != null) {
        message = failedPreconditionStatus;
      }
      break;
    case abortedStatusCode:
      if (abortedStatus != null) {
        message = abortedStatus;
      }
    case outOfRangeStatusCode:
      if (outOfRangeStatus != null) {
        message = outOfRangeStatus;
      }
      break;
    case unimplementedStatusCode:
      if (unimplementedStatus != null) {
        message = unimplementedStatus;
      }
      break;
    case internalStatusCode:
      if (internalStatus != null) {
        message = internalStatus;
      } else {
        message = l10n.serverError;
      }
      break;
    case unavailableStatusCode:
      if (unavailableStatus != null) {
        message = unavailableStatus;
      } else {
        message = l10n.serverStatusUnderMaintenance;
      }
      break;
    case dataLossStatusCode:
      if (dataLossStatus != null) {
        message = dataLossStatus;
      }
    case unauthenticatedStatusCode:
      if (unauthenticatedStatus != null) {
        message = unauthenticatedStatus;
      }
      break;
    default:
      break;
  }
  try {
    if (message is String && message.isNotEmpty) {
      rootScaffoldMessengerKey.currentState!
          .showSnackBar(SnackBar(content: Text(message)));
    } else if (message is Map) {
      rootScaffoldMessengerKey.currentState!
          .showSnackBar(SnackBar(content: Text(message[errorMessage])));
    }
  } catch (e) {
    logger.w("showResultMessage error: $e");
  }
}

/// 应用程序样式常量
class AppStyles {
  // 间距
  static const double smallPadding = 5.0;
  static const double defaultPadding = 8.0;
  static const double mediumPadding = 10.0;
  static const double largePadding = 16.0;

  // 圆角
  static const double defaultBorderRadius = 10.0;

  // 字体大小
  static const double smallFontSize = 14.0;
  static const double defaultFontSize = 16.0;
  static const double titleFontSize = 20.0;
  static const double largeFontSize = 25.0;

  // 图标大小
  static const double smallIconSize = 20.0;
  static const double defaultIconSize = 24.0;

  // 头像尺寸
  static const double smallAvatarSize = 20.0;
  static const double defaultAvatarSize = 40.0;
  static const double largeAvatarSize = 100.0;

  // 卡片样式
  static BoxDecoration cardDecoration(BuildContext context) {
    return BoxDecoration(
      color: Theme.of(context).cardColor,
      borderRadius: BorderRadius.circular(defaultBorderRadius),
      boxShadow: [
        BoxShadow(
          color: Colors.black.withValues(alpha: 0.1),
          blurRadius: 4,
          offset: const Offset(0, 2),
        ),
      ],
    );
  }

  // 按钮样式
  static ButtonStyle defaultButtonStyle = ButtonStyle(
    shape: WidgetStateProperty.all(
      RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(defaultBorderRadius),
      ),
    ),
  );
}

class UserAvatar extends StatelessWidget {
  final String imageUrl;
  final double size;
  final VoidCallback? onTap;
  final bool showEditIcon;

  const UserAvatar({
    Key? key,
    required this.imageUrl,
    this.size = AppStyles.defaultAvatarSize,
    this.onTap,
    this.showEditIcon = false,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Stack(
        children: [
          SizedBox(
            width: size,
            height: size,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(size / 4),
              child: CachedNetworkImage(
                imageUrl: imageUrl,
                fit: BoxFit.cover,
                errorWidget: (context, url, error) {
                  return Icon(
                    Icons.account_circle,
                    size: size,
                    color: Theme.of(context).disabledColor,
                  );
                },
              ),
            ),
          ),
          if (showEditIcon)
            Positioned(
              right: 0,
              bottom: 0,
              child: Container(
                width: size * 0.3,
                height: size * 0.3,
                decoration: BoxDecoration(
                  color: Theme.of(context).cardColor,
                  borderRadius: BorderRadius.only(
                    topLeft: Radius.circular(AppStyles.smallPadding),
                  ),
                ),
                child: Icon(
                  Icons.edit,
                  size: AppStyles.smallIconSize,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

Future safeRequest(Function func, var args, Function onError,
    {bool rethrowError = false}) async {
  logger.d("safeRequest called $func with args: $args");
  bool retryFlag = false;
  while (true) {
    try {
      var res = await func(args);
      if (retryFlag) {
        logger.i("Request succeeded after retry");
      }
      return res;
    } on GrpcError catch (e) {
      if (e.code == resourceExhaustedStatusCode &&
          e.message == "HTTP connection completed with 429 instead of 200") {
        retryFlag = true;
        logger.w("Rate limit exceeded, sleeping for a while");
        await Future.delayed(Duration(milliseconds: 500));
      } else {
        logger.w("GrpcError caught: $e");
        onError(e);
        if (rethrowError) {
          rethrow;
        }
        return;
      }
    } catch (e) {
      logger.w("Error caught: $e");
      if (rethrowError) {
        rethrow;
      }
    }
  }
}

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

List analyzeVersionString(String version) {
  List<String> versionList = version.replaceAll("v", "").split(".");
  int latestX, latestY, latestZ;
  latestX = int.parse(versionList[0]);
  latestY = int.parse(versionList[1]);
  latestZ = int.parse(versionList[2].replaceAll(RegExp("-.*"), ""));
  String other = version.replaceAll("v$latestX.$latestY.$latestZ-", "");
  return [
    latestX,
    latestY,
    latestZ,
    other,
    other.contains("alpha"),
    other.contains("beta")
  ];
}

Future needUpdate(Uri source, bool acceptAlpha, bool acceptBeta) async {
  http.Response res = await http.get(source);
  var data = jsonDecode(res.body);
  for (int i = 0; i < data.length; i++) {
    String? version = data[i]["tag_name"];
    if (version == null) return null;
    if (version == currentVersion) return null;
    List latestVersionList = analyzeVersionString(version);
    List currentVersionList = analyzeVersionString(currentVersion);
    for (int j = 0; j < 3; j++) {
      if (latestVersionList[j] > currentVersionList[j] &&
          (acceptAlpha || !latestVersionList[4]) &&
          (acceptBeta || !latestVersionList[5])) {
        return data[i];
      } else if (latestVersionList[j] < currentVersionList[j]) {
        return null;
      }
    }
    if (latestVersionList[4] && acceptAlpha) {
      return data[i];
    }
    if (latestVersionList[5] && acceptBeta) {
      return data[i];
    }
  }
  return null;
}

/// Replace urls in markdown text (GENERATE BY AI)
String replaceMarkdownImageUrls(
    String markdown, String Function(String oldUrl) replaceUrl) {
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

Future<UploadResponse> uploadStreaming(
    OurChatAppState ourchatAppState, Uint8List rawData, bool autoClean,
    {Int64? sessionId, bool compress = true, String? contentType}) async {
  var l10n = ourchatAppState.l10n;
  var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
      interceptors: [ourchatAppState.server!.interceptor!]);
  StreamController<UploadRequest> controller =
      StreamController<UploadRequest>();
  var call = safeRequest(stub.upload, controller.stream, (GrpcError e) {
    showResultMessage(
      ourchatAppState,
      e.code,
      e.message,
      invalidArgumentStatus: "${l10n.internalError}(${e.message})",
      resourceExhaustedStatus: l10n.storageSpaceFull,
    );
  }, rethrowError: true);
  Uint8List biData = rawData;
  if (compress) {
    biData = (await compressInQueue(ImageFileConfiguration(
      input:
          ImageFile(filePath: "", rawBytes: rawData, contentType: contentType),
    )))
        .rawBytes;
    logger.i(
        "upload: original size=${rawData.lengthInBytes}, compressed size=${biData.lengthInBytes}");
  }
  controller.add(UploadRequest(
    metadata: Header(
        hash: sha3_256.convert(biData.toList()).bytes,
        size: Int64.parseInt(biData.length.toString()),
        autoClean: autoClean,
        sessionId: sessionId),
  ));
  int chunkSize = 1024 * 128;
  for (int i = 0; i < biData.lengthInBytes; i += chunkSize) {
    logger.i(
        "upload: sending chunk ${i ~/ chunkSize + 1} of ${(biData.lengthInBytes / chunkSize).ceil()}");
    controller.add(UploadRequest(
        content: biData
            .sublist(
                i,
                biData.lengthInBytes > i + chunkSize
                    ? i + chunkSize
                    : biData.lengthInBytes)
            .toList()));
    logger.i("finish");
  }
  controller.close();
  return await call;
}

Future<UploadResponse> upload(
    OurChatAppState ourchatAppState, Uint8List rawData, bool autoClean,
    {Int64? sessionId, bool compress = true, String? contentType}) async {
  if (kIsWeb) {
    return await uploadChunked(ourchatAppState, rawData, autoClean,
        sessionId: sessionId, compress: compress, contentType: contentType);
  } else {
    return await uploadStreaming(ourchatAppState, rawData, autoClean,
        sessionId: sessionId, compress: compress, contentType: contentType);
  }
}

/// Chunked upload function for gRPC-web compatibility
/// Uses multiple unary RPC calls instead of streaming
Future<UploadResponse> uploadChunked(
    OurChatAppState ourchatAppState, Uint8List rawData, bool autoClean,
    {Int64? sessionId, bool compress = true, String? contentType}) async {
  var l10n = ourchatAppState.l10n;
  var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
      interceptors: [ourchatAppState.server!.interceptor!]);

  // Compress if needed
  Uint8List biData = rawData;
  if (compress) {
    biData = (await compressInQueue(ImageFileConfiguration(
      input:
          ImageFile(filePath: "", rawBytes: rawData, contentType: contentType),
    )))
        .rawBytes;
    logger.i(
        "uploadChunked: original size=${rawData.lengthInBytes}, compressed size=${biData.lengthInBytes}");
  }

  // Calculate hash
  var hash = sha3_256.convert(biData.toList()).bytes;

  // Start upload session
  var startResp = await safeRequest(
    stub.startUpload,
    StartUploadRequest(
      hash: hash,
      size: Int64.parseInt(biData.length.toString()),
      autoClean: autoClean,
      sessionId: sessionId,
    ),
    (GrpcError e) {
      showResultMessage(
        ourchatAppState,
        e.code,
        e.message,
        resourceExhaustedStatus: l10n.storageSpaceFull,
      );
    },
    rethrowError: true,
  );

  String uploadId = startResp.uploadId;
  int chunkSize = startResp.chunkSize.toInt();

  // Upload chunks sequentially
  int totalChunks = (biData.length / chunkSize).ceil();
  for (int i = 0; i < totalChunks; i++) {
    int start = i * chunkSize;
    int end =
        (start + chunkSize < biData.length) ? start + chunkSize : biData.length;
    var chunk = biData.sublist(start, end);

    var chunkResp = await safeRequest(
      stub.uploadChunk,
      UploadChunkRequest(
        uploadId: uploadId,
        chunkData: chunk,
        chunkId: Int64.parseInt(i.toString()),
      ),
      (GrpcError e) {
        showResultMessage(
          ourchatAppState,
          e.code,
          e.message,
          invalidArgumentStatus: "${l10n.internalError}(${e.message})",
        );
      },
      rethrowError: true,
    );

    logger.i(
        "Uploaded chunk ${i + 1}/$totalChunks (${chunkResp.bytesReceived}/${biData.length} bytes)");
  }

  // Complete upload
  var completeResp = await safeRequest(
    stub.completeUpload,
    CompleteUploadRequest(uploadId: uploadId),
    (GrpcError e) {
      showResultMessage(
        ourchatAppState,
        e.code,
        e.message,
        invalidArgumentStatus: "${l10n.internalError}(${e.message})",
      );
    },
    rethrowError: true,
  );

  return UploadResponse(key: completeResp.key);
}

/// Cancel an ongoing upload
Future<void> cancelUpload(
    OurChatAppState ourchatAppState, String uploadId) async {
  var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
      interceptors: [ourchatAppState.server!.interceptor!]);

  await safeRequest(
    stub.cancelUpload,
    CancelUploadRequest(uploadId: uploadId),
    (GrpcError e) {
      logger.e("Failed to cancel upload: $e");
    },
  );
}

Future<Uint8List> getOurChatFile(
    OurChatAppState ourchatAppState, String key) async {
  try {
    var manager = DefaultCacheManager();
    FileInfo? cache = await manager.getFileFromCache(
        "${ourchatAppState.server!.host}:${ourchatAppState.server!.port}/$key");
    if (cache != null) {
      return await cache.file.readAsBytes();
    }
    var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
        interceptors: [ourchatAppState.server!.interceptor!]);
    var res = await safeRequest(stub.download, DownloadRequest(key: key),
        (GrpcError e) {
      showResultMessage(ourchatAppState, e.code, e.message);
    }) as ResponseStream<DownloadResponse>;
    List<int> data = [];
    for (DownloadResponse piece in await res.toList()) {
      data.addAll(piece.data);
    }

    manager.putFile(
        "${ourchatAppState.server!.host}:${ourchatAppState.server!.port}/$key",
        Uint8List.fromList(data),
        key:
            "${ourchatAppState.server!.host}:${ourchatAppState.server!.port}/$key");
    return Uint8List.fromList(data);
  } catch (e) {
    logger.e("getOurChatFile(key:$key) error:$e");
    rethrow;
  }
}
