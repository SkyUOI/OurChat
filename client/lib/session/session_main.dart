import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/ui.dart';
import 'package:ourchat/main.dart';
import 'session_list.dart';
import 'session_tab.dart';

class Session extends ConsumerStatefulWidget {
  const Session({super.key});

  @override
  ConsumerState<Session> createState() => _SessionWidgetState();
}

class _SessionWidgetState extends ConsumerState<Session> {
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      // 此builder可以在尺寸发生变化时重新构建
      builder: (context, constraints) {
        if (ref.watch(screenModeProvider) == ScreenMode.desktop) {
          return Row(
            children: [
              Flexible(flex: 1, child: cardWithPadding(const SessionList())),
              Flexible(flex: 3, child: TabWidget()),
            ],
          );
        } else {
          return SessionList();
        }
      },
    );
  }
}
