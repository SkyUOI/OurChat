import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fixnum/fixnum.dart';
import 'package:ourchat/core/account.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/core/instance.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/server_setting.dart';
import 'package:ourchat/session.dart';
import 'package:ourchat/setting.dart';
import 'package:ourchat/friends.dart';
import 'package:ourchat/user.dart';

class Home extends ConsumerStatefulWidget {
  const Home({super.key});

  @override
  ConsumerState<Home> createState() => _HomeState();
}

class _HomeState extends ConsumerState<Home> {
  int index = 0;
  @override
  Widget build(BuildContext context) {
    final thisAccountId = ref.watch(thisAccountIdProvider);
    if (thisAccountId == null) {
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

    return Scaffold(
      body: SafeArea(
        child: LayoutBuilder(
          builder: (context, constraints) {
            if (ref.watch(screenModeProvider) == ScreenMode.mobile) {
              return Column(
                children: [
                  const AccountSwitcher(),
                  Expanded(
                    child: Padding(
                      padding: EdgeInsets.all(AppStyles.mediumPadding),
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
                        label: "Friends",
                        icon: Icon(Icons.people),
                      ),
                      BottomNavigationBarItem(
                        label: "Settings",
                        icon: Icon(Icons.settings),
                      ),
                      BottomNavigationBarItem(
                        label: "Me",
                        icon: Icon(Icons.person),
                      ),
                    ],
                    currentIndex: index,
                    onTap: (value) {
                      setState(() {
                        index = value;
                      });
                    },
                    type: BottomNavigationBarType.fixed,
                  ),
                ],
              );
            }
            return Row(
              children: [
                NavigationRail(
                  leading: const Padding(
                    padding: EdgeInsets.symmetric(vertical: 8.0),
                    child: AccountSwitcher(),
                  ),
                  destinations: const [
                    NavigationRailDestination(
                      label: Text("Sessions"),
                      icon: Icon(Icons.chat),
                    ),
                    NavigationRailDestination(
                      label: Text("Friends"),
                      icon: Icon(Icons.people),
                    ),
                    NavigationRailDestination(
                      label: Text("Settings"),
                      icon: Icon(Icons.settings),
                    ),
                    NavigationRailDestination(
                      label: Text("Me"),
                      icon: Icon(Icons.person),
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
                    padding: EdgeInsets.all(AppStyles.mediumPadding),
                    child: page,
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

/// Sentinel menu value meaning "add another server/account".
final _addServerKey = AccountKey('__add_server__', Int64(0));

/// A compact switcher that shows the active account and lets the user switch
/// between all concurrently-logged-in accounts (one per server/account), or
/// add another server's account.
class AccountSwitcher extends ConsumerWidget {
  const AccountSwitcher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final activeKey = ref.watch(activeAccountProvider);
    final instances = ref.watch(instancesProvider);
    if (instances.isEmpty) return const SizedBox.shrink();
    final activeInst = activeKey == null ? null : instances[activeKey];

    String nameFor(OurChatInstance inst) {
      final acc = ref.read(
        ourChatAccountProvider(inst.serverId, inst.accountId),
      );
      return acc.username.isNotEmpty ? acc.username : inst.accountId.toString();
    }

    String labelFor(OurChatInstance inst) {
      final label = _serverLabelOf(ref, inst.serverId);
      return '${nameFor(inst)} · $label';
    }

    return PopupMenuButton<AccountKey>(
      tooltip: 'Switch account',
      onSelected: (key) {
        if (key == _addServerKey) {
          Navigator.push(
            context,
            MaterialPageRoute(builder: (_) => const ServerSetting()),
          );
          return;
        }
        switchActive(ref, key);
      },
      itemBuilder: (context) => [
        for (final inst in instances.values)
          PopupMenuItem<AccountKey>(
            value: inst.key,
            child: Row(
              children: [
                Icon(
                  inst.key == activeKey
                      ? Icons.radio_button_checked
                      : Icons.radio_button_unchecked,
                  size: 18,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(labelFor(inst), overflow: TextOverflow.ellipsis),
                ),
              ],
            ),
          ),
        const PopupMenuDivider(),
        PopupMenuItem<AccountKey>(
          value: _addServerKey,
          child: Row(
            children: [
              const Icon(Icons.add, size: 18),
              const SizedBox(width: 8),
              Text(l10n.addServer),
            ],
          ),
        ),
      ],
      child: Padding(
        padding: EdgeInsets.all(AppStyles.smallPadding),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.account_circle, size: 32),
            const SizedBox(height: 4),
            ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 140),
              child: Text(
                activeInst == null ? '' : nameFor(activeInst),
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: AppStyles.smallFontSize),
              ),
            ),
            const Icon(Icons.arrow_drop_down, size: 16),
          ],
        ),
      ),
    );
  }

  String _serverLabelOf(WidgetRef ref, String serverId) {
    final cfg = ref.read(configProvider);
    for (final s in cfg.servers) {
      if (s.uniqueIdentifier == serverId) {
        if (s.label != null && s.label!.isNotEmpty) return s.label!;
        return '${s.host}:${s.port}';
      }
    }
    return serverId;
  }
}
