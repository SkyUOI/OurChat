import 'dart:typed_data';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/main.dart';
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
      List<UserMsg> records = await ref
          .read(ourChatEventSystemProvider.notifier)
          .getSessionEvent(
            sessionState.currentSessionId!,
            offset: 50 * sessionState.recordLoadCnt,
          );
      if (records.isEmpty ||
          sessionState.currentSessionRecords.contains(records.first)) {
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
  @override
  Widget build(BuildContext context) {
    UserMsg msg = widget.msg;
    double opacity = widget.opacity;
    var sessionState = ref.watch(sessionProvider);
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final senderData = msg.senderId != null
        ? ref.read(ourChatAccountProvider(msg.senderId!))
        : null;
    final senderNotifier = msg.senderId != null
        ? ref.read(ourChatAccountProvider(msg.senderId!).notifier)
        : null;
    final dn = senderData?.displayName;
    String name = dn != null && dn.isNotEmpty
        ? dn
        : (senderData?.username ?? "");
    bool isMe = msg.senderId != null && msg.senderId == thisAccountId;
    Widget avatar = UserAvatar(imageUrl: senderNotifier?.avatarUrl() ?? "");
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
              if (sessionState.cacheFiles.containsKey(uri.toString())) {
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
              }
              try {
                String content = uri.toString().split("://")[1];
                if (uri.scheme[0] == 'i') {
                  if (uri.scheme[1] == 'o') {
                    widget = FutureBuilder(
                      future: getOurChatFile(
                        ref,
                        msg.involvedFiles[int.parse(content)],
                      ),
                      builder: (content, snapshot) {
                        if (snapshot.hasError) {
                          return Text(
                            l10n.failTo("${l10n.load} ${l10n.image}"),
                          );
                        }
                        if (snapshot.connectionState != ConnectionState.done ||
                            snapshot.data == null) {
                          return CircularProgressIndicator(
                            color: Theme.of(context).primaryColor,
                          );
                        }
                        Uint8List fileBytes = snapshot.data as Uint8List;
                        return Image.memory(fileBytes);
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
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: // 根据是否为本账号的发言决定左右对齐
          (isMe
              ? MainAxisAlignment.end
              : MainAxisAlignment.start),
          children: [(isMe ? message : avatar), (isMe ? avatar : message)],
        ),
      ),
    );
  }
}
