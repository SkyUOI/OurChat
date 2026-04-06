import 'dart:convert';

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/service/ourchat/friends/accept_friend_invitation/v1/accept_friend_invitation.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:ourchat/session.dart';
import 'main.dart';

class Friends extends ConsumerWidget {
  const Friends({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final thisAccountData = ref.read(ourChatAccountProvider(thisAccountId!));
    return Column(
      children: [
        Flexible(
          flex: 1,
          child: ListView.builder(
            itemBuilder: (context, index) {
              return Padding(
                padding: EdgeInsets.all(AppStyles.mediumPadding),
                child: ElevatedButton.icon(
                  style: AppStyles.defaultButtonStyle,
                  icon: Icon(Icons.person_add),
                  onPressed: () {
                    showDialog(
                      context: context,
                      builder: (context) {
                        return FriendRequestDialog();
                      },
                    );
                  },
                  label: Text(l10n.friendRequest),
                ),
              );
            },
            itemCount: 1,
          ),
        ),
        Flexible(
          flex: 1,
          child: ListView.builder(
            itemBuilder: (context, index) {
              final friendId = thisAccountData.friends[index];
              final accountNotifier = ref.read(
                ourChatAccountProvider(friendId).notifier,
              );
              final accountData = ref.read(ourChatAccountProvider(friendId));
              accountNotifier.recreateStub();
              return Card(
                margin: EdgeInsets.symmetric(
                  vertical: AppStyles.smallPadding,
                  horizontal: AppStyles.mediumPadding,
                ),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(
                    AppStyles.defaultBorderRadius,
                  ),
                ),
                child: ListTile(
                  leading: FutureBuilder(
                    future: accountNotifier.getAccountInfo(),
                    builder: (context, snapshot) {
                      return UserAvatar(
                        imageUrl: accountNotifier.avatarUrl(),
                        size: AppStyles.smallAvatarSize,
                      );
                    },
                  ),
                  title: FutureBuilder(
                    future: accountNotifier.getAccountInfo(),
                    builder: (context, snapshot) {
                      if (snapshot.connectionState == ConnectionState.done) {
                        return Text(
                          accountData.displayName != null &&
                                  accountData.displayName!.isNotEmpty
                              ? accountData.displayName!
                              : accountData.username,
                          style: TextStyle(fontSize: AppStyles.defaultFontSize),
                        );
                      }
                      return Text(l10n.loading);
                    },
                  ),
                  onTap: () {
                    showDialog(
                      context: context,
                      builder: (context) {
                        return AlertDialog(
                          title: Text(accountData.username),
                          content: SizedBox(
                            width: 150,
                            child: Consumer(
                              builder: (context, ref, _) {
                                ref
                                    .read(sessionProvider.notifier)
                                    .openUserTab(accountData.id, l10n.userInfo);
                                return Column(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [UserTab()],
                                );
                              },
                            ),
                          ),
                        );
                      },
                    );
                  },
                ),
              );
            },
            itemCount: thisAccountData.friends.length,
          ),
        ),
        if (thisAccountData.friends.isEmpty)
          Flexible(flex: 1, child: Text(l10n.noFriend)),
      ],
    );
  }
}

class FriendRequestDialog extends ConsumerWidget {
  const FriendRequestDialog({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    final thisAccountNotifier = ref.read(
      ourChatAccountProvider(thisAccountId!).notifier,
    );
    return AlertDialog(
      title: Text(l10n.friendRequest),
      content: SizedBox(
        height: 300,
        width: 150,
        child: FutureBuilder(
          future: ref
              .read(ourChatEventSystemProvider.notifier)
              .selectNewFriendInvitation(),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.done) {
              List<NewFriendInvitationNotification> data = snapshot.data;
              return ListView.builder(
                itemBuilder: (context, index) {
                  final senderId = data[index].senderId!;
                  final inviteeId = data[index].inviteeId!;
                  final senderData = ref.read(ourChatAccountProvider(senderId));
                  final inviteeData = ref.read(
                    ourChatAccountProvider(inviteeId),
                  );
                  return Column(
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            (senderId == thisAccountId
                                ? inviteeData.username
                                : senderData.username),
                            textAlign: TextAlign.left,
                          ),
                          if (data[index].data!["status"] != 0)
                            Text(
                              data[index].data!["status"] == 1
                                  ? l10n.accepted
                                  : l10n.refused,
                              style: TextStyle(color: Colors.grey),
                            ),
                          if (data[index].data!["status"] == 0 &&
                              Int64.parseInt(
                                    data[index].data!["invitee"].toString(),
                                  ) ==
                                  thisAccountId)
                            Row(
                              children: [
                                IconButton(
                                  onPressed: () async {
                                    var server = ref.watch(
                                      ourChatServerProvider,
                                    );
                                    var stub = OurChatServiceClient(
                                      server.channel,
                                      interceptors: [server.interceptor!],
                                    );
                                    Navigator.pop(context);

                                    await safeRequest(
                                      stub.acceptFriendInvitation,
                                      AcceptFriendInvitationRequest(
                                        friendId: senderId,
                                        status: AcceptFriendInvitationResult
                                            .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS,
                                      ),
                                      (grpc.GrpcError e) {
                                        showResultMessage(
                                          e.code,
                                          e.message,
                                          permissionDeniedStatus:
                                              AppLocalizations.of(
                                                context,
                                              )!.permissionDenied(
                                                "Accept friend invitation",
                                              ),
                                          notFoundStatus:
                                              AppLocalizations.of(
                                                context,
                                              )!.notFound(
                                                AppLocalizations.of(
                                                  context,
                                                )!.invitation,
                                              ),
                                        );
                                      },
                                    );
                                    await thisAccountNotifier.getAccountInfo(
                                      ignoreCache: true,
                                    );
                                    await ref
                                        .read(
                                          ourChatAccountProvider(
                                            senderId,
                                          ).notifier,
                                        )
                                        .getAccountInfo(ignoreCache: true);
                                  },
                                  icon: Icon(Icons.check),
                                ),
                                IconButton(
                                  onPressed: () {
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
                                                  decoration: InputDecoration(
                                                    label: Text(
                                                      l10n.friendRequest,
                                                    ),
                                                  ),
                                                  onSaved: (newValue) async {
                                                    var server = ref.watch(
                                                      ourChatServerProvider,
                                                    );
                                                    var stub =
                                                        OurChatServiceClient(
                                                          server.channel,
                                                          interceptors: [
                                                            server.interceptor!,
                                                          ],
                                                        );
                                                    await safeRequest(
                                                      stub.acceptFriendInvitation,
                                                      AcceptFriendInvitationRequest(
                                                        friendId: senderId,
                                                        status: AcceptFriendInvitationResult
                                                            .ACCEPT_FRIEND_INVITATION_RESULT_FAIL,
                                                        leaveMessage: newValue,
                                                      ),
                                                      (grpc.GrpcError e) {
                                                        showResultMessage(
                                                          e.code,
                                                          e.message,
                                                          permissionDeniedStatus:
                                                              AppLocalizations.of(
                                                                context,
                                                              )!.permissionDenied(
                                                                "Refuse friend invitation",
                                                              ),
                                                          notFoundStatus:
                                                              AppLocalizations.of(
                                                                context,
                                                              )!.notFound(
                                                                l10n.invitation,
                                                              ),
                                                        );
                                                      },
                                                    );
                                                  },
                                                ),
                                              ),
                                            ],
                                          ),
                                          actions: [
                                            TextButton(
                                              onPressed: () {
                                                key.currentState!.save();
                                              },
                                              child: Text(l10n.ok),
                                            ),
                                            TextButton(
                                              onPressed: () {
                                                Navigator.pop(context);
                                              },
                                              child: Text(l10n.cancel),
                                            ),
                                          ],
                                        );
                                      },
                                    );
                                  },
                                  icon: Icon(Icons.close),
                                ),
                              ],
                            ),
                        ],
                      ),
                      if (data[index].leaveMessage != "" &&
                          data[index].data!["status"] != 2)
                        Align(
                          alignment: Alignment.centerLeft,
                          child: Text(
                            data[index].leaveMessage!,
                            textAlign: TextAlign.left,
                            style: TextStyle(fontSize: 10, color: Colors.grey),
                          ),
                        ),
                      if (data[index].data!["status"] == 2)
                        Align(
                          alignment: Alignment.centerLeft,
                          child: FutureBuilder(
                            future: getRefuseReason(data[index]),
                            builder: (context, snapshot) {
                              String text = l10n.loading;
                              if (snapshot.connectionState ==
                                  ConnectionState.done) {
                                text = snapshot.data;
                              }
                              return Text(
                                text,
                                textAlign: TextAlign.left,
                                style: TextStyle(
                                  fontSize: 10,
                                  color: Colors.grey,
                                ),
                              );
                            },
                          ),
                        ),
                      Divider(),
                    ],
                  );
                },
                itemCount: data.length,
              );
            }
            return Center(
              child: SizedBox(
                width: 50,
                height: 50,
                child: CircularProgressIndicator(
                  color: Theme.of(context).primaryColor,
                ),
              ),
            );
          },
        ),
      ),
    );
  }

  Future getRefuseReason(NewFriendInvitationNotification eventObj) async {
    String refuseReason = "";
    if (eventObj.status == 2) {
      // 已拒绝
      var event =
          await (privateDB!.select(privateDB!.record)..where(
                (u) => u.eventId.equals(
                  BigInt.from(eventObj.resultEventId!.toInt()),
                ),
              ))
              .getSingle();
      refuseReason = jsonDecode(event.data)["leave_message"];
    }
    return refuseReason;
  }
}
