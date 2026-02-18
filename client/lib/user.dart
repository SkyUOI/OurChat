import 'dart:typed_data';
import 'package:grpc/grpc.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/about.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:image_picker/image_picker.dart';
import 'package:ourchat/core/chore.dart';
import 'main.dart';

class User extends StatelessWidget {
  const User({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.mediumPadding),
          child: UserAvatar(
            imageUrl: ourchatAppState.thisAccount!.avatarUrl(),
            size: AppStyles.largeAvatarSize,
            showEditIcon: true,
            onTap: () async {
              ImagePicker picker = ImagePicker();
              XFile? image =
                  await picker.pickImage(source: ImageSource.gallery);
              if (image == null) return;
              Uint8List biData = await image.readAsBytes();
              var stub = OurChatServiceClient(ourchatAppState.server!.channel!,
                  interceptors: [ourchatAppState.server!.interceptor!]);
              try {
                showResultMessage(
                  ourchatAppState,
                  okStatusCode,
                  null,
                  okStatus: l10n.uploading,
                );
                var res = await upload(ourchatAppState, biData, false);
                showResultMessage(
                  ourchatAppState,
                  okStatusCode,
                  null,
                );
                await safeRequest(
                    stub.setSelfInfo, SetSelfInfoRequest(avatarKey: res.key),
                    (GrpcError e) {
                  showResultMessage(ourchatAppState, e.code, e.message,
                      invalidArgumentStatus: {
                        "Ocid Too Long": l10n.tooLong(l10n.ocid),
                        "Status Too Long": l10n.tooLong(l10n.status),
                      },
                      alreadyExistsStatus: l10n.alreadyExists(l10n.info));
                });
                await ourchatAppState.thisAccount!
                    .getAccountInfo(ignoreCache: true);
                ourchatAppState.update();
              } catch (e) {
                showResultMessage(ourchatAppState, internalStatusCode, null,
                    internalStatus: l10n.failTo(l10n.upload));
              }
            },
          ),
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 50,
            ),
            Card(
                elevation: 2,
                shape: RoundedRectangleBorder(
                  borderRadius:
                      BorderRadius.circular(AppStyles.defaultBorderRadius),
                ),
                margin: EdgeInsets.all(AppStyles.mediumPadding),
                child: Padding(
                    padding: EdgeInsets.all(AppStyles.mediumPadding),
                    child: Column(
                      children: [
                        Text(
                          ourchatAppState.thisAccount!.username,
                          style: TextStyle(
                              fontSize: AppStyles.titleFontSize,
                              fontWeight: FontWeight.bold),
                        ),
                        SizedBox(height: AppStyles.smallPadding),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Text("${l10n.email}: ",
                                style: TextStyle(color: Colors.grey)),
                            SelectableText(ourchatAppState.thisAccount!.email!),
                          ],
                        ),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Text("${l10n.ocid}: "),
                            SelectableText(ourchatAppState.thisAccount!.ocid),
                          ],
                        ),
                      ],
                    ))),
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
                                    initialValue:
                                        ourchatAppState.thisAccount!.username,
                                    decoration: InputDecoration(
                                        label: Text(l10n.username)),
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
                                    initialValue:
                                        ourchatAppState.thisAccount!.ocid,
                                    decoration:
                                        InputDecoration(label: Text(l10n.ocid)),
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
                              )),
                          actions: [
                            IconButton(
                                onPressed: () async {
                                  if (key.currentState!.validate()) {
                                    key.currentState!.save();
                                    var stub = OurChatServiceClient(
                                        ourchatAppState.server!.channel!,
                                        interceptors: [
                                          ourchatAppState.server!.interceptor!
                                        ]);

                                    await safeRequest(
                                        stub.setSelfInfo,
                                        SetSelfInfoRequest(
                                            userName: username,
                                            ocid: ocid), (GrpcError e) {
                                      showResultMessage(
                                          ourchatAppState, e.code, e.message,
                                          invalidArgumentStatus: {
                                            "Ocid Too Long":
                                                l10n.tooLong(l10n.ocid),
                                            "Status Too Long":
                                                l10n.tooLong(l10n.status),
                                          },
                                          alreadyExistsStatus:
                                              l10n.alreadyExists(l10n.info));
                                    });
                                    await ourchatAppState.thisAccount!
                                        .getAccountInfo(ignoreCache: true);
                                    ourchatAppState.update();
                                    if (context.mounted) {
                                      Navigator.pop(context);
                                    }
                                  }
                                },
                                icon: Icon(Icons.check)),
                            IconButton(
                                onPressed: () => Navigator.pop(context),
                                icon: Icon(Icons.close)),
                          ],
                        );
                      });
                },
                icon: Icon(Icons.edit))
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
                    ourchatAppState.thisAccount = null;
                    ourchatAppState.server = null;
                    ourchatAppState.privateDB!.close();
                    ourchatAppState.privateDB = null;
                    ourchatAppState.accountCachePool = {};
                    ourchatAppState.sessionCachePool = {};
                    ourchatAppState.eventSystem!.stopListening();
                    Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => ServerSetting()));
                  },
                  label: Text(l10n.logout)),
            ),
            Padding(
              padding: EdgeInsets.all(AppStyles.smallPadding),
              child: ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.info_outline),
                  onPressed: () {
                    Navigator.push(context,
                        MaterialPageRoute(builder: (context) => About()));
                  },
                  label: Text(l10n.about)),
            )
          ],
        ),
      ],
    );
  }
}
