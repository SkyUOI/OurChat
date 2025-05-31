import 'package:flutter/material.dart';
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
        ListView.builder(
          itemBuilder: (context, index) {
            return ElevatedButton(onPressed: () {}, child: Text("好友请求"));
          },
          itemCount: 1,
        ),
        ListView.builder(
            itemBuilder: (context, index) {
              return ElevatedButton(
                  onPressed: () {},
                  child: Text(
                      ourchatAppState.thisAccount!.friends[index].toString()));
            },
            itemCount: ourchatAppState.thisAccount!.friends.length)
      ],
    );
  }
}
