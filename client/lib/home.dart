import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session.dart';
import 'package:ourchat/setting.dart';
import 'package:provider/provider.dart';
import 'friends.dart';

class Home extends StatefulWidget {
  const Home({super.key});

  @override
  State<Home> createState() => _HomeState();
}

class _HomeState extends State<Home> {
  int index = 0;
  @override
  Widget build(BuildContext context) {
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    Widget page = const Placeholder();
    switch (index) {
      case 0:
        page = const Session();
        break;
      case 1:
        page = const Setting();
        break;
      case 2:
        page = const Friends();
        break;
    }

    return Scaffold(body: LayoutBuilder(builder: (context, constraints) {
      if (ourchatAppState.device == mobile) {
        return Column(
          children: [
            Expanded(
                child: Padding(
              padding: const EdgeInsets.all(8.0),
              child: page,
            )),
            BottomNavigationBar(
              items: const [
                BottomNavigationBarItem(
                    label: "Sessions", icon: Icon(Icons.chat)),
                BottomNavigationBarItem(
                    label: "Settings", icon: Icon(Icons.settings)),
                BottomNavigationBarItem(
                    label: "Friends", icon: Icon(Icons.people)),
              ],
              currentIndex: index,
              onTap: (value) {
                setState(() {
                  index = value;
                });
              },
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
                  label: Text("Settings"), icon: Icon(Icons.settings)),
              NavigationRailDestination(
                  label: Text("Friends"), icon: Icon(Icons.people))
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
            padding: const EdgeInsets.all(8.0),
            child: page,
          )),
        ],
      );
    }));
  }
}
