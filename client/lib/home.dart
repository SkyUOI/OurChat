import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/session.dart';
import 'package:ourchat/setting.dart';
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
    OurchatAppState ourchatAppState = context.watch<OurchatAppState>();
    Widget page = const Placeholder();
    if (index == 0) {
      page = const Session();
    } else if (index == 1) {
      page = const Setting();
    } else if (index == 2) {}

    return Scaffold(
      body: LayoutBuilder(
        builder: (context, constraints) {
          if (ourchatAppState.device == mobile) {
            return Column(
              children: [
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.all(8.0),
                    child: page,
                  ),
                ),
                BottomNavigationBar(
                  items: const [
                    BottomNavigationBarItem(
                      label: "Sessions",
                      icon: Icon(Icons.chat),
                    ),
                    BottomNavigationBarItem(
                      label: "Settings",
                      icon: Icon(Icons.settings),
                    ),
                    BottomNavigationBarItem(
                      label: "New",
                      icon: Icon(Icons.add),
                    ),
                  ],
                  currentIndex: index,
                  onTap: (value) {
                    setState(() {
                      index = value;
                    });
                  },
                ),
              ],
            );
          } else {
            return Row(
              children: [
                NavigationRail(
                  destinations: const [
                    NavigationRailDestination(
                      label: Text("Sessions"),
                      icon: Icon(Icons.chat),
                    ),
                    NavigationRailDestination(
                      label: Text("Settings"),
                      icon: Icon(Icons.settings),
                    ),
                    NavigationRailDestination(
                      label: Text("New"),
                      icon: Icon(Icons.add),
                    ),
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
                  ),
                ),
              ],
            );
          }
        },
      ),
    );
  }
}
