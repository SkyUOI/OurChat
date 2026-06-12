import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/friends/add_friend/v1/add_friend.pb.dart';
import 'package:ourchat/service/ourchat/friends/set_friend_info/v1/set_friend_info.pb.dart';

/// Full-page user profile view that appears when tapping a user's avatar.
/// Shows avatar, username, OCID, email (if visible), and friend actions.
class UserProfilePage extends ConsumerStatefulWidget {
  final Int64 userId;

  const UserProfilePage({super.key, required this.userId});

  @override
  ConsumerState<UserProfilePage> createState() => _UserProfilePageState();
}

class _UserProfilePageState extends ConsumerState<UserProfilePage> {
  Future<bool> _fetchAccountInfo() async {
    final notifier = ref.read(ourChatAccountProvider(widget.userId).notifier);
    notifier.recreateStub();
    return await notifier.getAccountInfo();
  }

  TableRow _infoRow(String field, String value) {
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

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);

    return Scaffold(
      appBar: AppBar(
        leading: BackButton(onPressed: () => Navigator.pop(context)),
        title: Text(l10n.userInfo),
      ),
      body: FutureBuilder(
        future: _fetchAccountInfo(),
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

          final accountData = ref.read(ourChatAccountProvider(widget.userId));
          final accountNotifier = ref.read(
            ourChatAccountProvider(widget.userId).notifier,
          );

          final currentAccountData = thisAccountId != null
              ? ref.read(ourChatAccountProvider(thisAccountId))
              : null;
          final isMe = thisAccountId == widget.userId;
          final isFriend = currentAccountData != null &&
              currentAccountData.friends.contains(widget.userId);

          return Center(
            child: SingleChildScrollView(
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
                        if (accountData.displayName != null &&
                            accountData.displayName!.isNotEmpty)
                          _infoRow(l10n.displayName, accountData.displayName!),
                        _infoRow(l10n.username, accountData.username),
                        _infoRow(l10n.ocid, accountData.ocid),
                        if (isMe || accountData.emailVisible)
                          _infoRow(l10n.email, accountData.email ?? ''),
                      ],
                    ),
                  ),
                  if (!isMe && !isFriend)
                    ElevatedButton.icon(
                      style: AppStyles.defaultButtonStyle,
                      icon: Icon(Icons.person_add),
                      onPressed: () => _showAddFriendDialog(context),
                      label: Text(l10n.addFriend),
                    ),
                  if (!isMe && isFriend)
                    ElevatedButton.icon(
                      style: AppStyles.defaultButtonStyle,
                      icon: Icon(Icons.edit),
                      onPressed: () =>
                          _showSetDisplayNameDialog(context),
                      label: Text(l10n.modify),
                    ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  void _showAddFriendDialog(BuildContext context) {
    String leaveMessage = "", displayName = "";

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
                    leaveMessage = newValue!;
                  },
                ),
                TextFormField(
                  decoration: InputDecoration(label: Text(l10n.displayName)),
                  onSaved: (newValue) {
                    displayName = newValue!;
                  },
                ),
              ],
            ),
          ),
          actions: [
            ElevatedButton.icon(
              style: AppStyles.defaultButtonStyle,
              icon: Icon(Icons.close),
              onPressed: () => Navigator.pop(context),
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
                      friendId: widget.userId,
                      displayName: displayName,
                      leaveMessage: leaveMessage,
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
                } catch (_) {}
              },
              label: Text(l10n.send),
            ),
          ],
        );
      },
    );
  }

  void _showSetDisplayNameDialog(BuildContext context) {
    final accountData = ref.read(ourChatAccountProvider(widget.userId));
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
                          id: widget.userId,
                          displayName: newValue,
                        ),
                        (grpc.GrpcError e) {
                          showResultMessage(e.code, e.message);
                        },
                      );
                      showResultMessage(okStatusCode, null);
                      await ref
                          .read(ourChatAccountProvider(widget.userId).notifier)
                          .getAccountInfo(ignoreCache: true);
                    } catch (_) {}
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
              onPressed: () => key.currentState!.save(),
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
  }
}
