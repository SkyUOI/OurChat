import 'dart:async';
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
import 'package:ourchat/service/basic/preset_user_status/v1/preset_user_status.pb.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';
import 'package:ourchat/service/ourchat/set_account_info/v1/set_account_info.pb.dart';
import 'package:image_picker/image_picker.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/auth_notifier.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/core/notification_service.dart';
import 'main.dart';

/// Fetches the server's preset user status list (via `BasicService.
/// GetPresetUserStatus`). Kept as an overridable provider so tests can inject
/// canned presets without a live channel.
final Provider<Future<List<String>> Function()> presetUserStatusProvider =
    Provider<Future<List<String>> Function()>((ref) {
      return () async {
        final server = ref.read(ourChatServerProvider);
        final stub = BasicServiceClient(server.channel);
        final res = await stub.getPresetUserStatus(
          GetPresetUserStatusRequest(),
        );
        return res.contents;
      };
    });

/// Dialog editing the signed-in user's own info: username, OCID and the
/// user-defined status (with the server's preset statuses offered as chips).
class SelfInfoEditDialog extends ConsumerStatefulWidget {
  const SelfInfoEditDialog({super.key, required this.accountData});

  final AccountData accountData;

  @override
  ConsumerState<SelfInfoEditDialog> createState() => _SelfInfoEditDialogState();
}

class _SelfInfoEditDialogState extends ConsumerState<SelfInfoEditDialog> {
  final GlobalKey<FormState> _formKey = GlobalKey<FormState>();
  late final TextEditingController _statusController = TextEditingController(
    text: widget.accountData.status ?? '',
  );
  // Session invitation privacy (issue #34): 0=everyone, 1=friends only,
  // 2=nobody — mirrors the server-side SessionInvitationPolicy enum.
  late int _invitePolicy = widget.accountData.sessionInvitationPolicy ?? 0;
  Future<List<String>>? _presetStatusesFuture;

  @override
  void initState() {
    super.initState();
    _presetStatusesFuture = ref.read(presetUserStatusProvider)();
  }

  @override
  void dispose() {
    _statusController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    String? username, ocid;
    return AlertDialog(
      content: Form(
        key: _formKey,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextFormField(
              initialValue: widget.accountData.username,
              decoration: InputDecoration(label: Text(l10n.username)),
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
              initialValue: widget.accountData.ocid,
              decoration: InputDecoration(label: Text(l10n.ocid)),
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
            TextFormField(
              controller: _statusController,
              decoration: InputDecoration(
                label: Text(l10n.status),
                helperMaxLines: 2,
              ),
              maxLength: 128,
            ),
            DropdownButtonFormField<int>(
              initialValue: _invitePolicy,
              decoration: InputDecoration(label: Text(l10n.invitePolicy)),
              items: [
                DropdownMenuItem(
                  value: 0,
                  child: Text(l10n.invitePolicyAllowAll),
                ),
                DropdownMenuItem(
                  value: 1,
                  child: Text(l10n.invitePolicyFriendsOnly),
                ),
                DropdownMenuItem(value: 2, child: Text(l10n.invitePolicyNobody)),
              ],
              onChanged: (value) {
                setState(() {
                  _invitePolicy = value ?? 0;
                });
              },
            ),
            FutureBuilder<List<String>>(
              future: _presetStatusesFuture,
              builder: (context, snapshot) {
                if (!snapshot.hasData || snapshot.data!.isEmpty) {
                  return const SizedBox.shrink();
                }
                return Align(
                  alignment: Alignment.centerLeft,
                  child: Padding(
                    padding: const EdgeInsets.only(top: 4.0, bottom: 8.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          l10n.presetStatus,
                          style: const TextStyle(
                            color: Colors.grey,
                            fontSize: 12,
                          ),
                        ),
                        const SizedBox(height: 4),
                        Wrap(
                          spacing: 6,
                          runSpacing: 4,
                          children: [
                            for (final preset in snapshot.data!)
                              InputChip(
                                label: Text(preset),
                                visualDensity: VisualDensity.compact,
                                onPressed: () {
                                  _statusController.text = preset;
                                },
                              ),
                          ],
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ],
        ),
      ),
      actions: [
        IconButton(
          onPressed: () async {
            if (_formKey.currentState!.validate()) {
              _formKey.currentState!.save();
              final status = _statusController.text.trim();
              var stub = ref.watch(ourChatServerProvider).newStub();

              await safeRequest(
                stub.setSelfInfo,
                SetSelfInfoRequest(
                  userName: username,
                  ocid: ocid,
                  userDefinedStatus: status,
                  sessionInvitationPolicy: _invitePolicy,
                ),
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
              final thisAccountId = ref.read(thisAccountIdProvider);
              final serverId = ref.read(activeServerIdProvider);
              if (thisAccountId != null && serverId != null) {
                await ref
                    .read(
                      ourChatAccountProvider(serverId, thisAccountId).notifier,
                    )
                    .getAccountInfo(ignoreCache: true);
              }
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
  }
}

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
                    if ((thisAccountData.status ?? '').isNotEmpty) ...[
                      SizedBox(height: AppStyles.smallPadding),
                      Text(
                        thisAccountData.status!,
                        style: TextStyle(
                          color: Colors.grey,
                          fontSize: AppStyles.defaultFontSize,
                        ),
                      ),
                    ],
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
                  builder: (context) =>
                      SelfInfoEditDialog(accountData: thisAccountData),
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
                  // Attention signals of the logged-out account must not
                  // outlive the session (issue #199).
                  unawaited(
                    ref.read(ourChatNotificationServiceProvider).cancelAll(),
                  );
                  stopFlashTray();
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
