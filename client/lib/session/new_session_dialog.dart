import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart' as grpc;
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/service/ourchat/session/new_session/v1/session.pb.dart';
import 'state.dart';

class NewSessionDialog extends ConsumerStatefulWidget {
  final SessionState sessionState;
  const NewSessionDialog({super.key, required this.sessionState});

  @override
  ConsumerState<NewSessionDialog> createState() => _NewSessionDialogState();
}

class _NewSessionDialogState extends ConsumerState<NewSessionDialog> {
  List<Int64> friends = [];
  List<bool> checked = [];
  bool gotFriendList = false, enableE2EE = true;

  void getFriendList() async {
    friends = [];
    final thisAccountId = ref.read(thisAccountIdProvider);
    final currentAccountData = ref.read(ourChatAccountProvider(thisAccountId!));
    for (int i = 0; i < currentAccountData.friends.length; i++) {
      Int64 friendId = currentAccountData.friends[i];
      final friendNotifier = ref.read(
        ourChatAccountProvider(friendId).notifier,
      );
      friendNotifier.recreateStub();
      await friendNotifier.getAccountInfo();
      friends.add(friendId);
    }
    for (int i = 0; i < friends.length; i++) {
      checked.add(false);
    }
    gotFriendList = true;
  }

  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    if (!gotFriendList) {
      getFriendList();
    }
    return AlertDialog(
      content: SizedBox(
        height: 450,
        width: 300,
        child: ListView.builder(
          itemBuilder: (context, index) {
            return SizedBox(
              height: 60.0,
              child: Padding(
                padding: const EdgeInsets.only(top: 10.0),
                child: ElevatedButton(
                  style: ButtonStyle(
                    shape: WidgetStateProperty.all(
                      RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10.0),
                      ),
                    ),
                  ),
                  onPressed: () {
                    setState(() {
                      checked[index] = !checked[index];
                    });
                  },
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.start,
                    children: [
                      SizedBox(
                        height: 40,
                        width: 40,
                        child: UserAvatar(
                          imageUrl: ref
                              .read(
                                ourChatAccountProvider(friends[index]).notifier,
                              )
                              .avatarUrl(),
                        ),
                      ),
                      Expanded(
                        child: Padding(
                          padding: EdgeInsets.only(left: 8.0),
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Align(
                                alignment: Alignment.centerLeft,
                                child: Text(
                                  ref
                                      .read(
                                        ourChatAccountProvider(friends[index]),
                                      )
                                      .username,
                                  style: TextStyle(
                                    fontSize: 20,
                                    color: Colors.black,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                      Checkbox(
                        value: checked[index],
                        onChanged: (v) {
                          setState(() {
                            checked[index] = v!;
                          });
                        },
                      ),
                    ],
                  ),
                ),
              ),
            );
          },
          itemCount: friends.length,
        ),
      ),
      actionsAlignment: MainAxisAlignment.spaceBetween,
      actions: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GestureDetector(
              child: Text(l10n.enableE2EE),
              onTap: () {
                setState(() {
                  enableE2EE = !enableE2EE;
                });
              },
            ),
            Checkbox(
              value: enableE2EE,
              onChanged: (v) {
                setState(() {
                  enableE2EE = v!;
                });
              },
            ),
          ],
        ),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              onPressed: () async {
                List<Int64> members = [];
                for (int i = 0; i < friends.length; i++) {
                  if (checked[i]) {
                    members.add(friends[i]);
                  }
                }
                var stub = ref.watch(ourChatServerProvider).newStub();
                try {
                  await safeRequest(
                    stub.newSession,
                    NewSessionRequest(members: members, e2eeOn: enableE2EE),
                    (grpc.GrpcError e) {
                      showResultMessage(
                        e.code,
                        e.message,
                        notFoundStatus: l10n.notFound(l10n.user),
                      );
                    },
                    rethrowError: true,
                  );
                  await ref
                      .read(ourChatAccountProvider(thisAccountId!).notifier)
                      .getAccountInfo(ignoreCache: true);
                  await ref.read(sessionProvider.notifier).loadSessions();
                } catch (e) {
                  return;
                }
                if (context.mounted) {
                  Navigator.pop(context);
                }
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
        ),
      ],
    );
  }
}
