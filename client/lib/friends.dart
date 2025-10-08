import 'dart:convert';

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/event.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/service/ourchat/friends/accept_friend_invitation/v1/accept_friend_invitation.pb.dart';
import 'package:ourchat/service/ourchat/v1/ourchat.pbgrpc.dart';
import 'package:ourchat/session.dart';
import 'package:provider/provider.dart';
import 'main.dart';

class Friends extends StatelessWidget {
  const Friends({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    OurChatAppState ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context);
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
                          });
                    },
                    label: Text(l10n!.friendRequest)),
              );
            },
            itemCount: 1,
          ),
        ),
        Flexible(
          flex: 1,
          child: ListView.builder(
              itemBuilder: (context, index) {
                var account = OurChatAccount(ourchatAppState);
                account.id = ourchatAppState.thisAccount!.friends[index];
                account.recreateStub();
                return Card(
                  margin: EdgeInsets.symmetric(
                    vertical: AppStyles.smallPadding,
                    horizontal: AppStyles.mediumPadding,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius:
                        BorderRadius.circular(AppStyles.defaultBorderRadius),
                  ),
                  child: ListTile(
                    leading: FutureBuilder(
                        future: account.getAccountInfo(),
                        builder: (context, snapshot) {
                          return UserAvatar(
                            imageUrl: account.avatarUrl(),
                            size: AppStyles.smallAvatarSize,
                          );
                        }),
                    title: FutureBuilder(
                        future: account.getAccountInfo(),
                        builder: (context, snapshot) {
                          if (snapshot.connectionState ==
                              ConnectionState.done) {
                            return Text(
                              account.displayName!.isNotEmpty
                                  ? account.displayName!
                                  : account.username,
                              style: TextStyle(
                                  fontSize: AppStyles.defaultFontSize),
                            );
                          }
                          return Text(l10n!.loading);
                        }),
                    onTap: () {
                      showDialog(
                          context: context,
                          builder: (context) {
                            return AlertDialog(
                              title: Text(account.username),
                              content: SizedBox(
                                width: 150,
                                child: ChangeNotifierProvider(
                                  create: (context) => SessionState(),
                                  builder: (context, child) {
                                    var sessionState =
                                        context.watch<SessionState>();
                                    sessionState.currentUserId = account.id;
                                    sessionState.tabIndex = userTab;
                                    return Column(
                                      mainAxisSize: MainAxisSize.min,
                                      children: [UserTab()],
                                    );
                                  },
                                ),
                              ),
                            );
                          });
                    },
                  ),
                );
              },
              itemCount: ourchatAppState.thisAccount!.friends.length),
        ),
        if (ourchatAppState.thisAccount!.friends.isEmpty)
          Flexible(flex: 1, child: Text("你还没有好友哦"))
      ],
    );
  }
}

class FriendRequestDialog extends StatelessWidget {
  const FriendRequestDialog({super.key});

  @override
  Widget build(BuildContext context) {
    OurChatAppState ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context);
    return AlertDialog(
      title: Text(l10n!.friendRequest),
      content: SizedBox(
        height: 300,
        width: 150,
        child: FutureBuilder(
          future: ourchatAppState.eventSystem!.selectNewFriendInvitation(),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.done) {
              List<NewFriendInvitationNotification> data = snapshot.data;
              return ListView.builder(
                  itemBuilder: (context, index) {
                    return Column(
                      children: [
                        Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Text(
                                (data[index].sender!.id ==
                                        ourchatAppState.thisAccount!.id
                                    ? data[index].invitee!.username
                                    : data[index].sender!.username),
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
                                  Int64.parseInt(data[index]
                                          .data!["invitee"]
                                          .toString()) ==
                                      ourchatAppState.thisAccount!.id)
                                Row(
                                  children: [
                                    IconButton(
                                        onPressed: () async {
                                          var stub = OurChatServiceClient(
                                              ourchatAppState.server!.channel!,
                                              interceptors: [
                                                ourchatAppState
                                                    .server!.interceptor!
                                              ]);
                                          Navigator.pop(context);
                                          try {
                                            await stub.acceptFriendInvitation(
                                                AcceptFriendInvitationRequest(
                                                    friendId:
                                                        data[index].sender!.id,
                                                    status: AcceptFriendInvitationResult
                                                        .ACCEPT_FRIEND_INVITATION_RESULT_SUCCESS));
                                          } on grpc.GrpcError catch (e) {
                                            if (context.mounted) {
                                              showResultMessage(
                                                  context, e.code, e.message,
                                                  permissionDeniedStatus:
                                                      AppLocalizations.of(
                                                              context)!
                                                          .permissionDenied(
                                                              "Accept friend invitation"),
                                                  notFoundStatus:
                                                      AppLocalizations.of(
                                                              context)!
                                                          .notFound(
                                                              AppLocalizations.of(
                                                                      context)!
                                                                  .invitation));
                                            }
                                          }
                                          await ourchatAppState.thisAccount!
                                              .getAccountInfo(
                                                  ignoreCache: true);
                                          await data[index]
                                              .sender!
                                              .getAccountInfo(
                                                  ignoreCache: true);
                                          ourchatAppState.update();
                                        },
                                        icon: Icon(Icons.check)),
                                    IconButton(
                                        onPressed: () {
                                          showDialog(
                                              context: context,
                                              builder: (context) {
                                                var key =
                                                    GlobalKey<FormState>();
                                                return AlertDialog(
                                                  content: Column(
                                                    mainAxisSize:
                                                        MainAxisSize.min,
                                                    children: [
                                                      Form(
                                                        key: key,
                                                        child: TextFormField(
                                                          decoration: InputDecoration(
                                                              label: Text(l10n
                                                                  .friendRequest)),
                                                          onSaved:
                                                              (newValue) async {
                                                            var stub = OurChatServiceClient(
                                                                ourchatAppState
                                                                    .server!
                                                                    .channel!,
                                                                interceptors: [
                                                                  ourchatAppState
                                                                      .server!
                                                                      .interceptor!
                                                                ]);
                                                            try {
                                                              await stub.acceptFriendInvitation(AcceptFriendInvitationRequest(
                                                                  friendId: data[
                                                                          index]
                                                                      .sender!
                                                                      .id,
                                                                  status: AcceptFriendInvitationResult
                                                                      .ACCEPT_FRIEND_INVITATION_RESULT_FAIL,
                                                                  leaveMessage:
                                                                      newValue));
                                                            } on grpc
                                                            .GrpcError catch (e) {
                                                              if (context
                                                                  .mounted) {
                                                                showResultMessage(
                                                                    context,
                                                                    e.code,
                                                                    e.message,
                                                                    permissionDeniedStatus: AppLocalizations.of(
                                                                            context)!
                                                                        .permissionDenied(
                                                                            "Refuse friend invitation"),
                                                                    notFoundStatus: AppLocalizations.of(
                                                                            context)!
                                                                        .notFound(
                                                                            l10n.invitation));
                                                                Navigator.pop(
                                                                    context);
                                                              }
                                                            }
                                                          },
                                                        ),
                                                      )
                                                    ],
                                                  ),
                                                  actions: [
                                                    TextButton(
                                                        onPressed: () {
                                                          key.currentState!
                                                              .save();
                                                        },
                                                        child: Text(l10n.ok)),
                                                    TextButton(
                                                        onPressed: () {
                                                          Navigator.pop(
                                                              context);
                                                        },
                                                        child:
                                                            Text(l10n.cancel))
                                                  ],
                                                );
                                              });
                                        },
                                        icon: Icon(Icons.close))
                                  ],
                                )
                            ]),
                        if (data[index].leaveMessage != "" &&
                            data[index].data!["status"] != 2)
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Text(
                              data[index].leaveMessage!,
                              textAlign: TextAlign.left,
                              style:
                                  TextStyle(fontSize: 10, color: Colors.grey),
                            ),
                          ),
                        if (data[index].data!["status"] == 2)
                          Align(
                              alignment: Alignment.centerLeft,
                              child: FutureBuilder(
                                  future: getRefuseReason(
                                      ourchatAppState, data[index]),
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
                                          fontSize: 10, color: Colors.grey),
                                    );
                                  })),
                        Divider()
                      ],
                    );
                  },
                  itemCount: data.length);
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

  Future getRefuseReason(OurChatAppState ourchatAppState,
      NewFriendInvitationNotification eventObj) async {
    String refuseReason = "";
    if (eventObj.status == 2) {
      // 已拒绝
      var event = await (ourchatAppState.privateDB!
              .select(ourchatAppState.privateDB!.record)
            ..where((u) =>
                u.eventId.equals(BigInt.from(eventObj.resultEventId!.toInt()))))
          .getSingle();
      refuseReason = jsonDecode(event.data)["leave_message"];
    }
    return refuseReason;
  }
}
