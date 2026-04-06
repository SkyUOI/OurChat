import 'dart:typed_data';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/about.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:image_picker/image_picker.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/auth_notifier.dart';
import 'package:ourchat/core/event.dart';
import 'main.dart';

class User extends ConsumerWidget {
  const User({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var thisAccountNotifier = ref.read(
      ourChatAccountProvider(thisAccountId!).notifier,
    );
    var thisAccountData = ref.read(ourChatAccountProvider(thisAccountId));
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.mediumPadding),
          child: UserAvatar(
            imageUrl: thisAccountNotifier.avatarUrl(),
            size: AppStyles.largeAvatarSize,
            showEditIcon: true,
            onTap: () async {
              ImagePicker picker = ImagePicker();
              XFile? image = await picker.pickImage(
                source: ImageSource.gallery,
              );
              if (image == null) return;
              Uint8List biData = await image.readAsBytes();
              var stub = ref.watch(ourChatServerProvider).newStub();
              try {
                showResultMessage(okStatusCode, null, okStatus: l10n.uploading);
                var res = await upload(
                  ref.watch(ourChatServerProvider),
                  biData,
                  false,
                );
                showResultMessage(okStatusCode, null);
                await safeRequest(
                  stub.setSelfInfo,
                  SetSelfInfoRequest(avatarKey: res.key),
                  (GrpcError e) {
                    showResultMessage(
                      e.code,
                      e.message,
                      invalidArgumentStatus: {
                        "Ocid Too Long": l10n.tooLong(l10n.ocid),
                        "Status Too Long": l10n.tooLong(l10n.status),
                      },
                      alreadyExistsStatus: l10n.alreadyExists(l10n.info),
                    );
                  },
                );
                await thisAccountNotifier.getAccountInfo(ignoreCache: true);
              } catch (e) {
                showResultMessage(
                  internalStatusCode,
                  null,
                  internalStatus: l10n.failTo(l10n.upload),
                );
              }
            },
          ),
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(width: 50),
            Card(
              elevation: 2,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(
                  AppStyles.defaultBorderRadius,
                ),
              ),
              margin: EdgeInsets.all(AppStyles.mediumPadding),
              child: Padding(
                padding: EdgeInsets.all(AppStyles.mediumPadding),
                child: Column(
                  children: [
                    Text(
                      thisAccountData.username,
                      style: TextStyle(
                        fontSize: AppStyles.titleFontSize,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    SizedBox(height: AppStyles.smallPadding),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(
                          "${l10n.email}: ",
                          style: TextStyle(color: Colors.grey),
                        ),
                        SelectableText(thisAccountData.email!),
                      ],
                    ),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text("${l10n.ocid}: "),
                        SelectableText(thisAccountData.ocid),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            IconButton(
              onPressed: () {
                showDialog(
                  context: context,
                  builder: (context) {
                    var key = GlobalKey<FormState>();
                    String? username, ocid;
                    return AlertDialog(
                      content: Form(
                        key: key,
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            TextFormField(
                              initialValue: thisAccountData.username,
                              decoration: InputDecoration(
                                label: Text(l10n.username),
                              ),
                              validator: (value) {
                                if (value!.isEmpty) {
                                  return l10n.cantBeEmpty;
                                }
                                return null;
                              },
                              onSaved: (newValue) {
                                username = newValue!;
                              },
                            ),
                            TextFormField(
                              initialValue: thisAccountData.ocid,
                              decoration: InputDecoration(
                                label: Text(l10n.ocid),
                              ),
                              validator: (value) {
                                if (value!.isEmpty) {
                                  return l10n.cantBeEmpty;
                                }
                                return null;
                              },
                              onSaved: (newValue) {
                                ocid = newValue!;
                              },
                            ),
                          ],
                        ),
                      ),
                      actions: [
                        IconButton(
                          onPressed: () async {
                            if (key.currentState!.validate()) {
                              key.currentState!.save();
                              var stub = ref
                                  .watch(ourChatServerProvider)
                                  .newStub();

                              await safeRequest(
                                stub.setSelfInfo,
                                SetSelfInfoRequest(
                                  userName: username,
                                  ocid: ocid,
                                ),
                                (GrpcError e) {
                                  showResultMessage(
                                    e.code,
                                    e.message,
                                    invalidArgumentStatus: {
                                      "Ocid Too Long": l10n.tooLong(l10n.ocid),
                                      "Status Too Long": l10n.tooLong(
                                        l10n.status,
                                      ),
                                    },
                                    alreadyExistsStatus: l10n.alreadyExists(
                                      l10n.info,
                                    ),
                                  );
                                },
                              );
                              await thisAccountNotifier.getAccountInfo(
                                ignoreCache: true,
                              );
                              if (context.mounted) {
                                Navigator.pop(context);
                              }
                            }
                          },
                          icon: Icon(Icons.check),
                        ),
                        IconButton(
                          onPressed: () => Navigator.pop(context),
                          icon: Icon(Icons.close),
                        ),
                      ],
                    );
                  },
                );
              },
              icon: Icon(Icons.edit),
            ),
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Padding(
              padding: EdgeInsets.all(AppStyles.smallPadding),
              child: ElevatedButton.icon(
                style: AppStyles.defaultButtonStyle,
                icon: Icon(Icons.logout),
                onPressed: () {
                  ref.read(authProvider.notifier).logout();
                  privateDB!.close();
                  privateDB = null;
                  ref.read(ourChatEventSystemProvider.notifier).stopListening();
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => ServerSetting()),
                  );
                },
                label: Text(l10n.logout),
              ),
            ),
            Padding(
              padding: EdgeInsets.all(AppStyles.smallPadding),
              child: ElevatedButton.icon(
                style: AppStyles.defaultButtonStyle,
                icon: Icon(Icons.info_outline),
                onPressed: () {
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => About()),
                  );
                },
                label: Text(l10n.about),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
