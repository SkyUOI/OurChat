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
          padding: const EdgeInsets.all(10.0),
          child: InkWell(
            child: Stack(
              children: [
                SizedBox(
                  width: 100.0,
                  height: 100.0,
                  child: Image(
                      image: NetworkImage(appState.thisAccount!.avatarUrl())),
                ),
                Positioned(
                    right: 0,
                    bottom: 0,
                    child: Container(
                      width: 30,
                      height: 30,
                      decoration: BoxDecoration(
                        backgroundBlendMode: BlendMode.overlay,
                        color: Theme.of(context).cardColor,
                        borderRadius:
                            BorderRadius.only(topLeft: Radius.circular(5)),
                      ),
                      child: Icon(Icons.edit, size: 20),
                    ))
              ],
            ),
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
              var call = stub.upload(controller.stream);
              controller.add(UploadRequest(
                metadata: Header(
                    hash: sha3_256.convert(biData.toList()).bytes,
                    size: Int64.parseInt(biData.length.toString()),
                    autoClean: false),
              ));
              controller.add(UploadRequest(content: biData.toList()));
              controller.close();
              var res = await call;

              await stub.setSelfInfo(SetSelfInfoRequest(avatarKey: res.key));
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
            Column(
              children: [
                Text(
                  appState.thisAccount!.username,
                  style: TextStyle(fontSize: 20),
                ),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text("${l10n.email}: "),
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
                                    await stub.setSelfInfo(SetSelfInfoRequest(
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
              padding: const EdgeInsets.all(5.0),
              child: ElevatedButton(
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
                  child: Text(l10n.logout)),
            ),
            Padding(
              padding: const EdgeInsets.all(5.0),
              child: ElevatedButton(
                  onPressed: () {
                    Navigator.push(context,
                        MaterialPageRoute(builder: (context) => About()));
                  },
                  child: Text(l10n.about)),
            )
          ],
        ),
      ],
    );
  }
}
