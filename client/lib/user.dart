import 'dart:async';
import 'dart:typed_data';

import 'package:hashlib/hashlib.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/about.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:ourchat/service/ourchat/upload/v1/upload.pb.dart';
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
    var appState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.all(AppStyles.mediumPadding),
          child: UserAvatar(
            imageUrl: appState.thisAccount!.avatarUrl(),
            size: AppStyles.largeAvatarSize,
            showEditIcon: true,
            onTap: () async {
              ImagePicker picker = ImagePicker();
              XFile? image =
                  await picker.pickImage(source: ImageSource.gallery);
              if (image == null) return;
              Uint8List biData = await image.readAsBytes();
              var stub = OurChatServiceClient(appState.server!.channel!,
                  interceptors: [appState.server!.interceptor!]);
              StreamController<UploadRequest> controller =
                  StreamController<UploadRequest>();
              var call = safeRequest(stub.upload, controller.stream);
              controller.add(UploadRequest(
                metadata: Header(
                    hash: sha3_256.convert(biData.toList()).bytes,
                    size: Int64.parseInt(biData.length.toString()),
                    autoClean: false),
              ));
              controller.add(UploadRequest(content: biData.toList()));
              controller.close();
              var res = await call;

              await safeRequest(
                  stub.setSelfInfo, SetSelfInfoRequest(avatarKey: res.key));
              await appState.thisAccount!.getAccountInfo(ignoreCache: true);
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
                          appState.thisAccount!.username,
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
                            SelectableText(appState.thisAccount!.email!),
                          ],
                        ),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Text("${l10n.ocid}: "),
                            SelectableText(appState.thisAccount!.ocid),
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
                                        appState.thisAccount!.username,
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
                                    initialValue: appState.thisAccount!.ocid,
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
                                        appState.server!.channel!,
                                        interceptors: [
                                          appState.server!.interceptor!
                                        ]);
                                    await safeRequest(
                                        stub.setSelfInfo,
                                        SetSelfInfoRequest(
                                            userName: username, ocid: ocid));
                                    await appState.thisAccount!
                                        .getAccountInfo(ignoreCache: true);
                                    appState.update();
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
                    appState.thisAccount = null;
                    appState.server = null;
                    appState.privateDB!.close();
                    appState.privateDB = null;
                    appState.accountCachePool = {};
                    appState.sessionCachePool = {};
                    appState.eventSystem!.stopListening();
                    Navigator.pop(context);
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
