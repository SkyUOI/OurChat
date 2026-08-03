import 'dart:convert';

import 'package:cached_network_image/cached_network_image.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/user_profile_page.dart';
import 'package:url_launcher/url_launcher.dart';
import 'state.dart';

class SessionRecord extends ConsumerStatefulWidget {
  const SessionRecord({super.key});

  @override
  ConsumerState<SessionRecord> createState() => _SessionRecordState();
}

class _SessionRecordState extends ConsumerState<SessionRecord> {
  ScrollController scrollController = ScrollController();

  @override
  void initState() {
    scrollController.addListener(onScroll);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    var sessionState = ref.watch(sessionProvider);
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var inputText = ref.watch(inputTextProvider);
    if (sessionState.recordLoadCnt != 1) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        scrollController.jumpTo(sessionState.lastPixels);
      });
    }
    return ListView.builder(
      controller: scrollController,
      itemBuilder: (context, index) {
        if (index == 0) {
          if (inputText.isEmpty) {
            return Container();
          }
          return MessageWidget(
            msg: UserMsg(senderId: thisAccountId, markdownText: inputText),
            opacity: 0.3,
          );
        } else {
          return MessageWidget(
            msg: sessionState.currentSessionRecords[index - 1],
            opacity: 1.0,
          );
        }
      },
      itemCount: sessionState.currentSessionRecords.length + 1,
      reverse: true,
    );
  }

  void onScroll() async {
    if (scrollController.position.maxScrollExtent -
            scrollController.position.pixels <
        300) {
      var sessionState = ref.read(sessionProvider);
      ref
          .read(sessionProvider.notifier)
          .setLastPixels(scrollController.position.pixels);

      // First try local DB
      final serverId = ref.read(activeServerIdProvider)!;
      final accountId = ref.read(activeAccountIdProvider)!;
      List<UserMsg> records = await ref
          .read(ourChatEventSystemProvider(serverId, accountId).notifier)
          .getSessionEvent(
            sessionState.currentSessionId!,
            offset: 50 * sessionState.recordLoadCnt,
          );

      // If local DB returns nothing, try server
      if (records.isEmpty && sessionState.currentSessionRecords.isNotEmpty) {
        final oldestMsg = sessionState.currentSessionRecords.last;
        final result = await ref
            .read(ourChatEventSystemProvider(serverId, accountId).notifier)
            .fetchSessionHistoryFromServer(
              sessionState.currentSessionId!,
              oldestMsg.sendTime!,
              limit: 50,
            );
        records = result.messages;
      }

      if (records.isEmpty ||
          (sessionState.currentSessionRecords.isNotEmpty &&
              sessionState.currentSessionRecords.contains(records.first))) {
        return;
      }
      ref.read(sessionProvider.notifier).addRecords(records);
    }
  }
}

class MessageWidget extends ConsumerStatefulWidget {
  final UserMsg msg;
  final double opacity;
  const MessageWidget({super.key, required this.msg, required this.opacity});

  @override
  ConsumerState<MessageWidget> createState() => _MessageWidgetState();
}

class _MessageWidgetState extends ConsumerState<MessageWidget> {
  UserMsg? _resolvedQuote;

  @override
  void initState() {
    super.initState();
    _resolveQuote();
  }

  @override
  void didUpdateWidget(covariant MessageWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.msg.quoteMsgId != widget.msg.quoteMsgId) {
      _resolvedQuote = null;
      _resolveQuote();
    }
  }

  /// For E2EE-sourced quotes the server cannot provide the quoted plaintext.
  /// Best-effort: resolve the quoted content from the local database.
  Future<void> _resolveQuote() async {
    final msg = widget.msg;
    if (msg.quoteMsgId == null) return;
    if (msg.quoteMarkdownText.isNotEmpty || msg.quoteInvolvedFiles.isNotEmpty) {
      return;
    }
    final key = ref.read(activeAccountProvider);
    final db = key == null ? null : ref.read(instancesProvider)[key]?.privateDB;
    if (db == null) return;
    try {
      final row =
          await (db.select(db.record)..where(
                (u) => u.eventId.equals(BigInt.from(msg.quoteMsgId!.toInt())),
              ))
              .getSingleOrNull();
      if (row == null || !mounted) return;
      final data = jsonDecode(row.data);
      final resolved = UserMsg(
        quoteMsgId: msg.quoteMsgId,
        quoteSenderId: Int64.parseInt(row.sender.toString()),
        quoteMarkdownText: data["markdown_text"] ?? "",
        quoteInvolvedFiles: [
          for (final f in (data["involved_files"] as List? ?? const []))
            f.toString(),
        ],
      );
      setState(() => _resolvedQuote = resolved);
    } catch (e) {
      // ignore resolution failures; the fallback placeholder is shown
    }
  }

  bool get _hasQuote => widget.msg.quoteMsgId != null;

  void _showQuoteMenu(Offset globalPosition) {
    showMenu(
      context: context,
      position: RelativeRect.fromLTRB(
        globalPosition.dx,
        globalPosition.dy,
        globalPosition.dx + 1,
        globalPosition.dy + 1,
      ),
      items: [
        PopupMenuItem(
          value: 'quote',
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.format_quote, size: 20),
              const SizedBox(width: 8),
              Text(l10n.quote),
            ],
          ),
        ),
      ],
    ).then((value) {
      if (value == 'quote') {
        ref.read(quoteTargetProvider.notifier).setQuote(widget.msg);
      }
    });
  }

  /// Render the quoted message as a compact block above the message body.
  Widget _buildQuoteBlock() {
    final msg = widget.msg;
    final resolved = _resolvedQuote;
    final quoteText = msg.quoteMarkdownText.isNotEmpty
        ? msg.quoteMarkdownText
        : (resolved?.quoteMarkdownText ?? '');
    final quoteFiles = msg.quoteInvolvedFiles.isNotEmpty
        ? msg.quoteInvolvedFiles
        : (resolved?.quoteInvolvedFiles ?? const []);
    final quoteSenderId = msg.quoteSenderId ?? resolved?.quoteSenderId;

    String quotedName = '';
    if (quoteSenderId != null) {
      final serverId = ref.read(activeServerIdProvider)!;
      final senderData = ref.read(
        ourChatAccountProvider(serverId, quoteSenderId),
      );
      final dn = senderData.displayName;
      quotedName = dn != null && dn.isNotEmpty ? dn : senderData.username;
    }

    String preview;
    if (quoteFiles.isNotEmpty && MarkdownToText.containsImage(quoteText)) {
      preview = '[${l10n.image}]';
    } else if (quoteText.isNotEmpty) {
      preview = MarkdownToText.convert(quoteText, l10n);
    } else {
      preview = l10n.quoteUnavailable;
    }

    return Container(
      margin: const EdgeInsets.only(bottom: 6.0),
      padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 6.0),
      constraints: const BoxConstraints(maxWidth: 260),
      decoration: BoxDecoration(
        color: Colors.grey.shade200,
        borderRadius: BorderRadius.circular(6.0),
        border: Border(left: BorderSide(color: Colors.grey.shade400, width: 3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            quotedName,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w600,
              color: Colors.grey.shade700,
            ),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 2),
          Text(
            preview,
            style: TextStyle(fontSize: 12, color: Colors.grey.shade700),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    UserMsg msg = widget.msg;
    double opacity = widget.opacity;
    var sessionState = ref.watch(sessionProvider);
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final serverId = ref.watch(activeServerIdProvider);
    final senderData = msg.senderId != null
        ? ref.read(ourChatAccountProvider(serverId!, msg.senderId!))
        : null;
    final senderNotifier = msg.senderId != null
        ? ref.read(ourChatAccountProvider(serverId!, msg.senderId!).notifier)
        : null;
    final dn = senderData?.displayName;
    String name = dn != null && dn.isNotEmpty
        ? dn
        : (senderData?.username ?? "");
    bool isMe = msg.senderId != null && msg.senderId == thisAccountId;
    Widget avatar = UserAvatar(
      imageUrl: senderNotifier?.avatarUrl() ?? "",
      onTap: msg.senderId != null && !isMe
          ? () {
              Navigator.push(
                context,
                MaterialPageRoute(
                  builder: (_) => UserProfilePage(userId: msg.senderId!),
                ),
              );
            }
          : null,
    );
    TextPainter textPainter = TextPainter(
      text: TextSpan(text: MarkdownToText.convert(msg.markdownText, l10n)),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout(
      maxWidth: ref.read(screenModeProvider) == ScreenMode.desktop
          ? 500.0
          : 250.0,
    );
    Widget message = Column(
      crossAxisAlignment: (isMe
          ? CrossAxisAlignment.end
          : CrossAxisAlignment.start),
      children: [
        Text(name),
        if (_hasQuote) _buildQuoteBlock(),
        ConstrainedBox(
          constraints: BoxConstraints(
            maxWidth:
                textPainter.width +
                (MarkdownToText.containsImage(msg.markdownText) ? 150.0 : 50.0),
          ),
          child: Markdown(
            selectable: true,
            softLineBreak: true,
            data: msg.markdownText,
            onTapLink: (text, href, title) {
              if (href == null) return;
              // Handle IO:// links for file downloads
              final parsedUri = Uri.tryParse(href);
              if (parsedUri != null &&
                  parsedUri.scheme.length >= 2 &&
                  parsedUri.scheme[0] == 'i' &&
                  parsedUri.scheme[1] == 'o') {
                final index = int.tryParse(
                  parsedUri.toString().split("://")[1],
                );
                if (index != null && index < msg.involvedFiles.length) {
                  // Trigger file download in background
                  getOurChatFile(ref, msg.involvedFiles[index])
                      .then((_) {
                        // File downloaded successfully
                      })
                      .catchError((_) {
                        showResultMessage(
                          internalStatusCode,
                          null,
                          internalStatus: l10n.failTo(
                            "${l10n.load} ${l10n.file}",
                          ),
                        );
                      });
                  return;
                }
              }
              showDialog(
                context: context,
                builder: (context) {
                  return AlertDialog(
                    title: Text(l10n.areUSure),
                    content: Text(l10n.toExternalWebsite(href)),
                    actions: [
                      IconButton(
                        onPressed: () {
                          Navigator.pop(context);
                          launchUrl(Uri.parse(href));
                        },
                        icon: Icon(Icons.check),
                      ),
                      IconButton(
                        onPressed: () {
                          Navigator.pop(context);
                        },
                        icon: Icon(Icons.close),
                      ),
                    ],
                  );
                },
              );
            },
            imageBuilder: (uri, title, alt) {
              Widget widget = Text(l10n.internalError);
              // Preview cached local files before upload
              if (sessionState.cacheFiles.containsKey(uri.toString())) {
                bool isImage =
                    sessionState.cacheFilesContentType[uri.toString()]
                        ?.startsWith('image/') ??
                    true;
                if (isImage) {
                  widget = InkWell(
                    onTap: () {
                      ref
                          .read(sessionProvider.notifier)
                          .switchSendRaw(uri.toString());
                    },
                    child: Stack(
                      children: [
                        Image.memory(sessionState.cacheFiles[uri.toString()]!),
                        if (sessionState.cacheFilesSendRaw[uri.toString()]!)
                          Icon(Icons.raw_on)
                        else
                          Icon(Icons.raw_off),
                      ],
                    ),
                  );
                } else {
                  // Non-image file preview
                  widget = _buildFileCard(
                    contentType:
                        sessionState.cacheFilesContentType[uri.toString()] ??
                        '',
                    filename: sessionState.cacheFileNames[uri.toString()] ?? '',
                    size: sessionState.cacheFiles[uri.toString()]!.length,
                    isLocal: true,
                  );
                }
              }
              try {
                String content = uri.toString().split("://")[1];
                if (uri.scheme[0] == 'i') {
                  if (uri.scheme[1] == 'o') {
                    widget = FutureBuilder<OurChatFileResult>(
                      future: getOurChatFile(
                        ref,
                        msg.involvedFiles[int.parse(content)],
                      ),
                      builder: (context, snapshot) {
                        if (snapshot.hasError) {
                          return Text(l10n.failTo("${l10n.load} ${l10n.file}"));
                        }
                        if (snapshot.connectionState != ConnectionState.done ||
                            snapshot.data == null) {
                          return CircularProgressIndicator(
                            color: Theme.of(context).primaryColor,
                          );
                        }
                        final result = snapshot.data!;
                        if (result.contentType.startsWith('image/')) {
                          return Image.memory(result.bytes);
                        } else {
                          return _buildFileCard(
                            contentType: result.contentType,
                            filename: result.filename,
                            size: result.size,
                          );
                        }
                      },
                    );
                  } else if (uri.scheme[1] == 'n') {
                    var path = content.split(",");
                    String url = "${path[0]}://${path.sublist(1).join(',')}";
                    widget = CachedNetworkImage(
                      imageUrl: url,
                      errorWidget: (context, url, error) => Text(
                        l10n.failTo("${l10n.load} ${l10n.image}($url) "),
                      ),
                    );
                  }
                }
              } catch (e) {
                // do nothing
              }
              return widget;
            },
            noScroll: true,
          ),
        ),
      ],
    );
    return Opacity(
      opacity: opacity,
      child: Container(
        margin: const EdgeInsets.all(5.0),
        decoration: BoxDecoration(),
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onLongPressStart: (details) => _showQuoteMenu(details.globalPosition),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: // Align left/right based on whether this is our own message
            (isMe
                ? MainAxisAlignment.end
                : MainAxisAlignment.start),
            children: [(isMe ? message : avatar), (isMe ? avatar : message)],
          ),
        ),
      ),
    );
  }

  /// Build a file attachment card for non-image files
  Widget _buildFileCard({
    required String contentType,
    required String filename,
    required int size,
    bool isLocal = false,
  }) {
    IconData fileIcon;
    if (contentType.startsWith('video/')) {
      fileIcon = Icons.videocam;
    } else if (contentType.startsWith('audio/')) {
      fileIcon = Icons.audiotrack;
    } else if (contentType.startsWith('image/')) {
      fileIcon = Icons.image;
    } else if (contentType.contains('pdf')) {
      fileIcon = Icons.picture_as_pdf;
    } else if (contentType.contains('zip') ||
        contentType.contains('tar') ||
        contentType.contains('compress')) {
      fileIcon = Icons.archive;
    } else {
      fileIcon = Icons.insert_drive_file;
    }

    String sizeStr;
    if (size < 1024) {
      sizeStr = '$size B';
    } else if (size < 1024 * 1024) {
      sizeStr = '${(size / 1024).toStringAsFixed(1)} KB';
    } else {
      sizeStr = '${(size / 1024 / 1024).toStringAsFixed(1)} MB';
    }

    String displayName = filename.isNotEmpty ? filename : l10n.file;
    if (displayName.length > 30) {
      displayName = '${displayName.substring(0, 27)}...';
    }

    return Container(
      width: 220,
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Colors.grey.shade100,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey.shade300),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(fileIcon, size: 40, color: Colors.grey.shade600),
          const SizedBox(height: 6),
          Text(
            displayName,
            style: TextStyle(fontWeight: FontWeight.w500, fontSize: 13),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 4),
          Text(
            sizeStr,
            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
          ),
        ],
      ),
    );
  }
}
