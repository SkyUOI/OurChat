import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session.dart';
import 'package:ourchat/setting.dart';
import 'package:ourchat/friends.dart';
import 'package:ourchat/user.dart';
import 'package:provider/provider.dart';

class Home extends StatefulWidget {
  const Home({super.key});

  @override
  State<Home> createState() => _HomeState();
}

class _HomeState extends State<Home> {
  int index = 0;
  @override
  Widget build(BuildContext context) {
    OurChatAppState ourchatAppState = context.watch<OurChatAppState>();
    if (ourchatAppState.thisAccount == null) {
      return Container();
    }
    Widget page = const Placeholder();
    switch (index) {
      case 0:
        page = const Session();
        break;
      case 1:
        page = const Friends();
        break;
      case 2:
        page = const Setting();
        break;
      case 3:
        page = const User();
    }

    return Scaffold(body: SafeArea(
      child: LayoutBuilder(builder: (context, constraints) {
        if (ourchatAppState.screenMode == mobile) {
          return Column(
            children: [
              Expanded(
                  child: Padding(
                padding: EdgeInsets.all(AppStyles.mediumPadding),
                child: page,
              )),
              BottomNavigationBar(
                items: const [
                  BottomNavigationBarItem(
                      label: "Sessions", icon: Icon(Icons.chat)),
                  BottomNavigationBarItem(
                      label: "Friends", icon: Icon(Icons.people)),
                  BottomNavigationBarItem(
                      label: "Settings", icon: Icon(Icons.settings)),
                  BottomNavigationBarItem(
                      label: "Me", icon: Icon(Icons.person)),
                ],
                currentIndex: index,
                onTap: (value) {
                  setState(() {
                    index = value;
                  });
                },
                type: BottomNavigationBarType.fixed,
              )
            ],
          );
        }
        return Row(
          children: [
            NavigationRail(
              destinations: const [
                NavigationRailDestination(
                    label: Text("Sessions"), icon: Icon(Icons.chat)),
                NavigationRailDestination(
                    label: Text("Friends"), icon: Icon(Icons.people)),
                NavigationRailDestination(
                    label: Text("Settings"), icon: Icon(Icons.settings)),
                NavigationRailDestination(
                    label: Text("Me"), icon: Icon(Icons.person)),
              ],
              selectedIndex: index,
              onDestinationSelected: (value) {
                setState(() {
                  index = value;
                });
              },
              labelType: NavigationRailLabelType.selected,
            ),
            Expanded(
                child: Padding(
              padding: EdgeInsets.all(AppStyles.mediumPadding),
              child: page,
            )),
          ],
        );
      }),
    ));
  }
}
