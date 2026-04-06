import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
import 'package:ourchat/service/ourchat/friends/set_friend_info/v1/set_friend_info.pb.dart';
import 'state.dart';

class UserTab extends ConsumerStatefulWidget {
  const UserTab({super.key});

  @override
  ConsumerState<UserTab> createState() => _UserTabState();
}

class _UserTabState extends ConsumerState<UserTab> {
  String addFriendLeaveMessage = "", addFriendDisplayName = "";

  Future<bool> fetchAccountInfo(Int64 id) async {
    final notifier = ref.read(ourChatAccountProvider(id).notifier);
    notifier.recreateStub();
    return await notifier.getAccountInfo();
  }

  TableRow userInfoRow(String field, String value) {
    return TableRow(
      children: [
        TableCell(
          child: Text(
            field,
            style: TextStyle(color: Colors.grey),
            textAlign: TextAlign.right,
          ),
        ),
        TableCell(child: Container()), // Spacer
        TableCell(child: Text(value, textAlign: TextAlign.left)),
      ],
    );
  }

  void showAddFriendDialog(BuildContext context, Int64 accountId) {
    final accountData = ref.read(ourChatAccountProvider(accountId));
    showDialog(
      context: context,
      builder: (context) {
        var formKey = GlobalKey<FormState>();
        return AlertDialog(
          content: Form(
            key: formKey,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextFormField(
                  decoration: InputDecoration(
                    label: Text(l10n.addFriendMessage),
                  ),
                  onSaved: (newValue) {
                    addFriendLeaveMessage = newValue!;
                  },
                ),
                TextFormField(
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) {
                    addFriendDisplayName = newValue!;
                  },
                ),
              ],
            ),
          ),
          actions: [
            ElevatedButton.icon(
              style: AppStyles.defaultButtonStyle,
              icon: Icon(Icons.close),
              onPressed: () {
                Navigator.pop(context);
              },
              label: Text(l10n.cancel),
            ),
            ElevatedButton.icon(
              style: AppStyles.defaultButtonStyle,
              icon: Icon(Icons.send),
              onPressed: () async {
                formKey.currentState!.save();
                var stub = ref.watch(ourChatServerProvider).newStub();
                Navigator.pop(context);
                try {
                  await safeRequest(
                    stub.addFriend,
                    AddFriendRequest(
                      friendId: accountData.id,
                      displayName: addFriendDisplayName,
                      leaveMessage: addFriendLeaveMessage,
                    ),
                    (grpc.GrpcError e) {
                      showResultMessage(
                        e.code,
                        e.message,
                        permissionDeniedStatus: l10n.permissionDenied(
                          l10n.addFriend,
                        ),
                        alreadyExistsStatus: l10n.friendAlreadyExists,
                        notFoundStatus: l10n.notFound(l10n.user),
                      );
                    },
                    rethrowError: true,
                  );
                  showResultMessage(okStatusCode, null);
                } catch (e) {
                  // do nothing
                }
              },
              label: Text(l10n.send),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    var sessionState = ref.watch(sessionProvider);
    final userId = sessionState.currentUserId!;
    return FutureBuilder(
      future: fetchAccountInfo(userId),
      builder: (context, snapshot) {
        if (snapshot.connectionState != ConnectionState.done) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                CircularProgressIndicator(
                  color: Theme.of(context).primaryColor,
                ),
                Text(l10n.loading),
              ],
            ),
          );
        }
        final accountData = ref.read(ourChatAccountProvider(userId));

        final accountNotifier = ref.read(
          ourChatAccountProvider(userId).notifier,
        );
        final currentAccountData = ref.read(
          ourChatAccountProvider(thisAccountId!),
        );
        bool isFriend = currentAccountData.friends.contains(accountData.id);
        return Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Padding(
                padding: EdgeInsets.all(AppStyles.mediumPadding),
                child: UserAvatar(
                  imageUrl: accountNotifier.avatarUrl(),
                  size: AppStyles.largeAvatarSize,
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(20.0),
                child: Table(
                  columnWidths: {
                    0: FlexColumnWidth(15),
                    1: FlexColumnWidth(1),
                    2: FlexColumnWidth(15),
                  },
                  children: [
                    if (accountData.displayName != null)
                      userInfoRow(l10n.displayName, accountData.displayName!),
                    userInfoRow(l10n.username, accountData.username),
                    userInfoRow(l10n.ocid, accountData.ocid),
                  ],
                ),
              ),
              if (!isFriend)
                ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.person_add),
                  onPressed: () => showAddFriendDialog(context, userId),
                  label: Text(l10n.addFriend),
                ),
              if (isFriend)
                ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.edit),
                  onPressed: () => showSetDisplayNameDialog(context, userId),
                  label: Text(l10n.modify),
                ),
            ],
          ),
        );
      },
    );
  }

  void showSetDisplayNameDialog(BuildContext context, Int64 accountId) {
    final accountData = ref.read(ourChatAccountProvider(accountId));
    showDialog(
      context: context,
      builder: (context) {
        var key = GlobalKey<FormState>();
        return AlertDialog(
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Form(
                key: key,
                child: TextFormField(
                  initialValue: accountData.displayName,
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) async {
                    var stub = ref.watch(ourChatServerProvider).newStub();

                    try {
                      await safeRequest(
                        stub.setFriendInfo,
                        SetFriendInfoRequest(
                          id: accountId,
                          displayName: newValue,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(e.code, e.message);
                        },
                      );

                      showResultMessage(okStatusCode, null);

                      await ref
                          .read(ourChatAccountProvider(accountId).notifier)
                          .getAccountInfo(ignoreCache: true);
                    } catch (e) {
                      // do nothing
                    }

                    if (context.mounted) {
                      Navigator.pop(context);
                    }
                  },
                ),
              ),
            ],
          ),
          actions: [
            IconButton(
              onPressed: () {
                key.currentState!.save();
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
  }
}
