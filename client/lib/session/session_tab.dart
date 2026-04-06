import 'dart:typed_data';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:image_picker/image_picker.dart';
import 'package:mime/mime.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/log.dart';
import 'package:ourchat/core/session.dart' as core_session;
import 'package:ourchat/core/ui.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/msg_delivery/v1/msg_delivery.pb.dart';
import 'package:ourchat/service/ourchat/session/delete_session/v1/delete_session.pb.dart';
import 'package:ourchat/service/ourchat/session/leave_session/v1/leave_session.pb.dart';
import 'package:ourchat/service/ourchat/session/set_session_info/v1/set_session_info.pb.dart';
import 'empty_tab.dart';
import 'session_record.dart';
import 'state.dart';
import 'user_tab.dart';

class SessionTab extends ConsumerStatefulWidget {
  const SessionTab({super.key});

  @override
  ConsumerState<SessionTab> createState() => _SessionTabState();
}

class _SessionTabState extends ConsumerState<SessionTab> {
  TextEditingController controller = TextEditingController();
  GlobalKey<FormState> inputBoxKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    var sessionState = ref.watch(sessionProvider);
    var key = GlobalKey<FormState>();

    return Form(
      key: key,
      child: Column(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Expanded(child: cardWithPadding(const SessionRecord())), //聊天记录
          Row(
            children: [
              Expanded(
                child: SizedBox(
                  height: 100,
                  child: cardWithPadding(
                    Align(
                      alignment: Alignment.bottomCenter,
                      child: SingleChildScrollView(
                        child: TextFormField(
                          key: inputBoxKey,
                          decoration: InputDecoration(hintText: "Type here..."),
                          maxLines: null,
                          validator: (value) {
                            if (value == null || value.isEmpty) {
                              return l10n.cantBeEmpty;
                            }
                            return null;
                          },
                          onSaved: (value) async {
                            List<String> involvedFiles = [];
                            String text = value!;
                            int index = 0;
                            if (sessionState.needUploadFiles.isNotEmpty) {
                              showResultMessage(
                                okStatusCode,
                                null,
                                okStatus: l10n.uploading,
                              );
                            }

                            for (String path in sessionState.needUploadFiles) {
                              try {
                                if (!sessionState.cacheFiles.containsKey(
                                  path,
                                )) {
                                  showResultMessage(
                                    notFoundStatusCode,
                                    null,
                                    notFoundStatus: l10n.notFound(
                                      "${l10n.image}($path)",
                                    ),
                                  );
                                  continue;
                                }
                                logger.i(
                                  "Uploading file: $path, compress: ${!sessionState.cacheFilesSendRaw[path]!}",
                                );

                                var res = await upload(
                                  ref.watch(ourChatServerProvider),
                                  sessionState.cacheFiles[path]!,
                                  true,
                                  sessionId: sessionState.currentSessionId!,
                                  compress:
                                      !sessionState.cacheFilesSendRaw[path]!,
                                  contentType:
                                      sessionState.cacheFilesContentType[path]!,
                                );

                                String newPath = "IO://$index";
                                text = replaceMarkdownImageUrls(text, (oldUrl) {
                                  if (oldUrl != path) {
                                    return oldUrl;
                                  }
                                  return newPath;
                                });
                                involvedFiles.add(res.key);
                                index += 1;
                              } catch (e) {
                                showResultMessage(
                                  internalStatusCode,
                                  null,
                                  internalStatus: l10n.failTo(l10n.upload),
                                );
                              }
                            }
                            if (sessionState.needUploadFiles.isNotEmpty) {
                              showResultMessage(okStatusCode, null);
                            }
                            var stub = ref
                                .read(ourChatServerProvider)
                                .newStub();
                            try {
                              await safeRequest(
                                stub.sendMsg,
                                SendMsgRequest(
                                  sessionId: sessionState.currentSessionId!,
                                  markdownText: text,
                                  involvedFiles: involvedFiles,
                                  isEncrypted: false,
                                ),
                                (grpc.GrpcError e) {
                                  showResultMessage(
                                    e.code,
                                    e.message,
                                    notFoundStatus: l10n.notFound(l10n.session),
                                    permissionDeniedStatus: l10n
                                        .permissionDenied(l10n.send),
                                  );
                                },
                              );
                            } catch (e) {
                              // do nothing
                            }
                            controller.text = "";
                            ref.read(inputTextProvider.notifier).setText("");
                            ref.read(sessionProvider.notifier).resetInputArea();
                          },
                          onChanged: (value) {
                            ref.read(inputTextProvider.notifier).setText(value);
                          },
                          controller: controller,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              Column(
                mainAxisAlignment: MainAxisAlignment.spaceAround,
                children: [
                  IconButton(
                    onPressed: () async {
                      var picker = ImagePicker();
                      List<XFile> images = await picker.pickMultiImage();
                      for (XFile i in images) {
                        var uri = Uri.parse(i.path);
                        var bytes = await i.readAsBytes();
                        var sendRaw = false;
                        var contentType = lookupMimeType(
                          i.path,
                          headerBytes: List.from(bytes),
                        )!;
                        var newCacheFiles = Map<String, Uint8List>.from(
                          sessionState.cacheFiles,
                        );
                        newCacheFiles[uri.toString()] = bytes;
                        var newSendRaw = Map<String, bool>.from(
                          sessionState.cacheFilesSendRaw,
                        );
                        newSendRaw[uri.toString()] = sendRaw;
                        var newContentTypes = Map<String, String>.from(
                          sessionState.cacheFilesContentType,
                        );
                        newContentTypes[uri.toString()] = contentType;
                        String breakLine =
                            controller.text.isEmpty ||
                                controller.text.endsWith("\n") // 已有换行
                            ? ""
                            : "\n";
                        controller.text =
                            "${controller.text}$breakLine![${i.name}](${uri.toString()})";
                        ref
                            .read(inputTextProvider.notifier)
                            .setText(controller.text);
                        ref
                            .read(sessionProvider.notifier)
                            .updateCacheFiles(
                              newCacheFiles,
                              newContentTypes,
                              newSendRaw,
                            );
                        ref
                            .read(sessionProvider.notifier)
                            .addNeedUploadFile(uri.toString());
                      }
                    },
                    icon: Icon(Icons.add),
                  ),
                  ElevatedButton.icon(
                    style: AppStyles.defaultButtonStyle,
                    onPressed: () {
                      if (key.currentState!.validate()) {
                        key.currentState!.save();
                      }
                    },
                    label: Text(l10n.send),
                    icon: Icon(Icons.send),
                  ),
                ],
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class TabWidget extends ConsumerStatefulWidget {
  const TabWidget({super.key});

  @override
  ConsumerState<TabWidget> createState() => _TabWidgetState();
}

class _TabWidgetState extends ConsumerState<TabWidget> {
  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    SessionState sessionState = ref.watch(sessionProvider);
    Widget tab;
    switch (sessionState.tabIndex) {
      case TabType.session:
        tab = SessionTab();
        break;
      case TabType.user:
        tab = UserTab();
        break;
      default:
        tab = EmptyTab();
        break;
    }
    Widget page = const Placeholder();
    // 匹配不同设备类型
    if (ref.watch(screenModeProvider) == ScreenMode.mobile) {
      page = SafeArea(
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    BackButton(
                      onPressed: () {
                        ref.read(sessionProvider.notifier).clearTab();
                        Navigator.pop(context);
                      },
                    ),
                    Text(sessionState.tabTitle, style: TextStyle(fontSize: 20)),
                  ],
                ),
                if (sessionState.tabIndex == TabType.session)
                  IconButton(
                    onPressed: () => showSetSessionInfoDialog(
                      context,
                      thisAccountId,
                      sessionState,
                    ),
                    icon: Icon(Icons.more_horiz),
                  ),
              ],
            ),
            Expanded(child: tab),
          ],
        ),
      );
    } else if (ref.watch(screenModeProvider) == ScreenMode.desktop) {
      page = Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Expanded(
                child: Center(
                  child: Text(
                    sessionState.tabTitle,
                    style: TextStyle(fontSize: 30),
                  ),
                ),
              ),
              if (sessionState.tabIndex == TabType.session)
                IconButton(
                  onPressed: () => showSetSessionInfoDialog(
                    context,
                    thisAccountId,
                    sessionState,
                  ),
                  icon: Icon(Icons.more_horiz),
                ),
            ],
          ),
          Expanded(child: tab),
        ],
      );
    }
    return Scaffold(body: page);
  }

  void showSetSessionInfoDialog(
    BuildContext context,
    Int64? thisAccountId,
    SessionState sessionState,
  ) {
    final sessionData = ref.read(
      core_session.ourChatSessionProvider(sessionState.currentSessionId!),
    );
    String name = sessionData.name, description = sessionData.description;
    var key = GlobalKey<FormState>();

    showDialog(
      context: context,
      builder: (BuildContext context) {
        bool confirmLeave = false;
        bool confirmDelete = false;
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Text(
                sessionData.name.isEmpty ? l10n.newSession : sessionData.name,
              ),
              content: Form(
                key: key,
                child: SizedBox(
                  width: 150,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Row(
                        children: [
                          Padding(
                            padding: const EdgeInsets.only(right: 5.0),
                            child: Text(l10n.sessionId),
                          ),
                          SelectableText(
                            sessionState.currentSessionId.toString(),
                          ),
                        ],
                      ),
                      TextFormField(
                        initialValue: name,
                        decoration: InputDecoration(
                          label: Text(l10n.sessionName),
                        ),
                        onSaved: (newValue) {
                          name = newValue!;
                        },
                      ),
                      TextFormField(
                        initialValue: description,
                        decoration: InputDecoration(
                          label: Text(l10n.description),
                        ),
                        onSaved: (newValue) {
                          description = newValue!;
                        },
                      ),
                    ],
                  ),
                ),
              ),
              actions: [
                if (sessionData.myPermissions.contains(deleteSessionPermission))
                  IconButton(
                    onPressed: () async {
                      if (!confirmDelete) {
                        setState(() {
                          confirmLeave = false;
                          confirmDelete = true;
                        });
                        rootScaffoldMessengerKey.currentState!.showSnackBar(
                          SnackBar(content: Text(l10n.againToConfirm)),
                        );
                        return;
                      }

                      var stub = ref.watch(ourChatServerProvider).newStub();
                      try {
                        safeRequest(
                          stub.deleteSession,
                          DeleteSessionRequest(
                            sessionId: sessionState.currentSessionId!,
                          ),
                          (grpc.GrpcError e) {
                            showResultMessage(
                              e.code,
                              e.message,
                              notFoundStatus: l10n.notFound(l10n.session),
                              permissionDeniedStatus: l10n.permissionDenied(
                                l10n.delete,
                              ),
                            );
                          },
                          rethrowError: true,
                        );
                        Navigator.pop(context);
                        showResultMessage(okStatusCode, null);
                        await ref
                            .read(
                              ourChatAccountProvider(thisAccountId!).notifier,
                            )
                            .getAccountInfo(ignoreCache: true);
                        await ref.read(sessionProvider.notifier).loadSessions();
                      } catch (e) {
                        // do nothing
                      }
                    },
                    icon: Icon(
                      Icons.delete_forever,
                      color: (confirmDelete ? Colors.redAccent : null),
                    ),
                  ),
                IconButton(
                  onPressed: () async {
                    if (!confirmLeave) {
                      setState(() {
                        confirmDelete = false;
                        confirmLeave = true;
                      });
                      rootScaffoldMessengerKey.currentState!.showSnackBar(
                        SnackBar(content: Text(l10n.againToConfirm)),
                      );
                      return;
                    }

                    var stub = ref.watch(ourChatServerProvider).newStub();
                    try {
                      safeRequest(
                        stub.leaveSession,
                        LeaveSessionRequest(
                          sessionId: sessionState.currentSessionId!,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(
                            e.code,
                            e.message,
                            notFoundStatus: l10n.notFound(l10n.session),
                          );
                        },
                      );
                      showResultMessage(okStatusCode, null);
                      // Navigator.pop(context);
                      await ref
                          .read(ourChatAccountProvider(thisAccountId!).notifier)
                          .getAccountInfo(ignoreCache: true);
                      await ref.read(sessionProvider.notifier).loadSessions();
                    } catch (e) {
                      // do nothing
                    }
                  },
                  icon: Icon(
                    Icons.exit_to_app,
                    color: (confirmLeave ? Colors.redAccent : null),
                  ),
                ),
                IconButton(
                  onPressed: () async {
                    key.currentState!.save();
                    var stub = ref.watch(ourChatServerProvider).newStub();

                    try {
                      await safeRequest(
                        stub.setSessionInfo,
                        SetSessionInfoRequest(
                          sessionId: sessionState.currentSessionId!,
                          name: name,
                          description: description,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(
                            e.code,
                            e.message,
                            alreadyExistsStatus: l10n.conflict,
                            permissionDeniedStatus: l10n.permissionDenied(
                              e.message!,
                            ),
                          );
                        },
                        rethrowError: true,
                      );
                      await ref
                          .read(
                            core_session
                                .ourChatSessionProvider(
                                  sessionState.currentSessionId!,
                                )
                                .notifier,
                          )
                          .getSessionInfo(ignoreCache: true);
                      setState(() {
                        final updatedData = ref.read(
                          core_session.ourChatSessionProvider(
                            sessionState.currentSessionId!,
                          ),
                        );
                        ref
                            .read(sessionProvider.notifier)
                            .updateTabTitle(updatedData.name);
                      });
                      showResultMessage(okStatusCode, null);
                    } catch (e) {
                      // do nothing
                    }
                    if (context.mounted) {
                      Navigator.pop(context);
                    }
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
    );
  }
}
