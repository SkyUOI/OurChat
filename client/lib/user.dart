import 'dart:typed_data';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/instance.dart';
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
    final serverId = ref.watch(activeServerIdProvider);
    var thisAccountNotifier = ref.read(
      ourChatAccountProvider(serverId!, thisAccountId!).notifier,
    );
    var thisAccountData = ref.read(
      ourChatAccountProvider(serverId, thisAccountId),
    );
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
                icon: Icon(Icons.add),
                onPressed: () {
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => ServerSetting()),
                  );
                },
                label: Text(l10n.addAccount),
              ),
            ),
            Padding(
              padding: EdgeInsets.all(AppStyles.smallPadding),
              child: ElevatedButton.icon(
                style: AppStyles.defaultButtonStyle,
                icon: Icon(Icons.logout),
                onPressed: () async {
                  final key = ref.read(activeAccountProvider);
                  if (key != null) {
                    ref
                        .read(
                          ourChatEventSystemProvider(
                            key.serverId,
                            key.accountId,
                          ).notifier,
                        )
                        .stopListening();
                    // Tear down the active instance: close its private DB and
                    // remove it from the registry.
                    final inst = ref.read(instancesProvider)[key];
                    if (inst != null) {
                      await inst.privateDB.close();
                    }
                    ref.read(instancesProvider.notifier).remove(key);
                  }
                  ref.read(activeAccountProvider.notifier).clear();
                  ref
                      .read(configProvider.notifier)
                      .setActiveAccount(null, null);
                  privateDB = null;
                  ref.read(authProvider.notifier).logout();
                  if (context.mounted) {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => ServerSetting()),
                    );
                  }
                },
                label: Text(l10n.logout),
              ),
            ),
            Padding(
              padding: EdgeInsets.all(AppStyles.smallPadding),
              child: ElevatedButton.icon(
                style: AppStyles.defaultButtonStyle,
                icon: Icon(Icons.manage_accounts),
                onPressed: () => _showAccountManager(context, ref),
                label: Text(l10n.account),
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

  String _serverLabelOf(WidgetRef ref, String serverId) {
    final cfg = ref.read(configProvider);
    for (final s in cfg.servers) {
      if (s.uniqueIdentifier == serverId) {
        if (s.label != null && s.label!.isNotEmpty) return s.label!;
        return '${s.host}:${s.port}';
      }
    }
    return serverId;
  }

  void _showAccountManager(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setState) {
            final config = ref.read(configProvider);
            final accounts = config.savedAccounts;
            final instances = ref.read(instancesProvider);
            return AlertDialog(
              title: Text(l10n.accountManager),
              content: SizedBox(
                width: 420,
                height: 320,
                child: accounts.isEmpty
                    ? Center(child: Text(l10n.noSavedAccount))
                    : ListView.builder(
                        itemCount: accounts.length,
                        itemBuilder: (context, i) {
                          final acc = accounts[i];
                          final online = instances.containsKey(
                            AccountKey(acc.serverId, Int64(acc.accountId)),
                          );
                          return ListTile(
                            dense: true,
                            leading: Icon(
                              online ? Icons.circle : Icons.circle_outlined,
                              size: 12,
                              color: online ? Colors.green : Colors.grey,
                            ),
                            title: Text(
                              acc.email ??
                                  acc.ocid ??
                                  l10n.accountId(acc.accountId),
                              overflow: TextOverflow.ellipsis,
                            ),
                            subtitle: Text(
                              '${_serverLabelOf(ref, acc.serverId)} (id: ${acc.accountId})',
                              overflow: TextOverflow.ellipsis,
                            ),
                            trailing: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(l10n.autoLogin),
                                Checkbox(
                                  value: acc.autoLogin,
                                  onChanged: (v) {
                                    ref
                                        .read(configProvider.notifier)
                                        .upsertSavedAccount(
                                          acc.copyWith(autoLogin: v ?? false),
                                        );
                                    setState(() {});
                                  },
                                ),
                                IconButton(
                                  icon: const Icon(Icons.delete_outline),
                                  onPressed: () {
                                    ref
                                        .read(configProvider.notifier)
                                        .removeSavedAccount(
                                          acc.serverId,
                                          acc.accountId,
                                        );
                                    setState(() {});
                                  },
                                ),
                              ],
                            ),
                          );
                        },
                      ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text(l10n.close),
                ),
              ],
            );
          },
        );
      },
    );
  }
}
