import 'package:flutter/material.dart';
import 'package:ourchat/core/event.dart';
import 'package:provider/provider.dart';
import 'main.dart';

class Friends extends StatelessWidget {
  const Friends({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    return Column(
      children: [
        Flexible(
          flex: 1,
          child: ListView.builder(
            itemBuilder: (context, index) {
              return ElevatedButton(
                  onPressed: () {
                    showDialog(
                        context: context,
                        builder: (context) {
                          return FriendRequestDialog();
                        });
                  },
                  child: Text("好友请求"));
            },
            itemCount: 1,
          ),
        ),
        Flexible(
          flex: 1,
          child: ListView.builder(
              itemBuilder: (context, index) {
                return ElevatedButton(
                    onPressed: () {},
                    child: Text(ourchatAppState.thisAccount!.friends[index]
                        .toString()));
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
  const FriendRequestDialog({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    return AlertDialog(
      title: Text("好友申请"),
      content: SizedBox(
        height: 300,
        width: 150,
        child: FutureBuilder(
          future: ourchatAppState.eventSystem!.selectFriendApproval(),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.done) {
              List<AddFriendApproval> data = snapshot.data;
              return ListView.builder(
                  itemBuilder: (context, index) {
                    return Column(
                      children: [
                        Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Text(
                                data[index].sender.username,
                                textAlign: TextAlign.left,
                              ),
                              Row(
                                children: [
                                  IconButton(
                                      onPressed: () {},
                                      icon: Icon(Icons.check)),
                                  IconButton(
                                      onPressed: () {}, icon: Icon(Icons.close))
                                ],
                              )
                            ]),
                        if (data[index].leaveMessage != "")
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Text(
                              data[index].leaveMessage,
                              textAlign: TextAlign.left,
                              style:
                                  TextStyle(fontSize: 10, color: Colors.grey),
                            ),
                          ),
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
}
