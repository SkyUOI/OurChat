import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/config.dart';
import 'package:ourchat/core/chore.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/auth.dart';
import 'package:ourchat/core/server.dart';

/// Server management page: lists all saved servers, lets the user add / edit /
/// delete them, and connects to one to reach the login/register screen.
class ServerSetting extends ConsumerStatefulWidget {
  const ServerSetting({super.key});

  @override
  ConsumerState<ServerSetting> createState() => _ServerSettingState();
}

class _ServerSettingState extends ConsumerState<ServerSetting> {
  bool _connectingId = false;

  Future<void> _connectServer(ServerConfig sc) async {
    setState(() {
      _connectingId = true;
    });
    final isTLS = sc.isTLS ?? await OurChatServer.tlsEnabled(sc.host, sc.port);
    final server = OurChatServer(sc.host, sc.port, isTLS);
    ref.read(ourChatServerProvider.notifier).update(server);
    final res = await server.getServerInfo();
    if (!mounted) return;
    setState(() {
      _connectingId = false;
    });
    if (res == unavailableStatusCode || res == unknownStatusCode) {
      showResultMessage(
        unavailableStatusCode,
        null,
        internalStatus: '${l10n.serverStatus}: ${l10n.serverStatusOffline}',
      );
      return;
    }
    // Persist the freshly-probed server info (uniqueIdentifier + isTLS).
    ref
        .read(configProvider.notifier)
        .upsertServer(
          ServerConfig(
            host: sc.host,
            port: sc.port,
            label: sc.label,
            uniqueIdentifier: server.uniqueIdentifier,
            isTLS: isTLS,
          ),
        );
    Navigator.pushReplacement(
      context,
      MaterialPageRoute(builder: (_) => Auth()),
    );
  }

  Future<void> _addOrEditServer([ServerConfig? existing]) async {
    final hostCtrl = TextEditingController(text: existing?.host ?? '');
    final portCtrl = TextEditingController(
      text: (existing?.port ?? 7777).toString(),
    );
    final labelCtrl = TextEditingController(text: existing?.label ?? '');

    final result = await showDialog<(String host, int port, String label)>(
      context: context,
      builder: (context) {
        final key = GlobalKey<FormState>();
        String host = existing?.host ?? '';
        int port = existing?.port ?? 7777;
        String label = existing?.label ?? '';
        return AlertDialog(
          title: Text(existing == null ? l10n.add : l10n.edit),
          content: Form(
            key: key,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextFormField(
                  controller: hostCtrl,
                  decoration: InputDecoration(label: Text(l10n.address)),
                  validator: (v) =>
                      (v == null || v.isEmpty) ? l10n.cantBeEmpty : null,
                  onSaved: (v) => host = v!,
                ),
                TextFormField(
                  controller: portCtrl,
                  decoration: InputDecoration(label: Text(l10n.port)),
                  validator: (v) {
                    final p = int.tryParse(v ?? '');
                    if (p == null || p < 0 || p > 65535) {
                      return l10n.invalid(l10n.port);
                    }
                    return null;
                  },
                  onSaved: (v) => port = int.parse(v!),
                ),
                TextFormField(
                  controller: labelCtrl,
                  decoration: InputDecoration(label: Text(l10n.remarkOptional)),
                  onSaved: (v) => label = v ?? '',
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: Text(l10n.cancel),
            ),
            TextButton(
              onPressed: () {
                if (key.currentState!.validate()) {
                  key.currentState!.save();
                  Navigator.pop(context, (host, port, label));
                }
              },
              child: Text(l10n.ok),
            ),
          ],
        );
      },
    );
    if (result == null || !mounted) return;
    final (host, port, label) = result;
    ref
        .read(configProvider.notifier)
        .upsertServer(
          ServerConfig(
            host: host,
            port: port,
            label: label,
            uniqueIdentifier: existing?.uniqueIdentifier,
            isTLS: existing?.isTLS,
          ),
        );
  }

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configProvider);
    final servers = config.servers;

    return Scaffold(
      appBar: AppBar(title: Text(l10n.selectServer)),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: servers.isEmpty
                  ? Center(child: Text(l10n.noServer))
                  : ListView.builder(
                      itemCount: servers.length,
                      itemBuilder: (context, i) {
                        final sc = servers[i];
                        final title = (sc.label != null && sc.label!.isNotEmpty)
                            ? sc.label!
                            : '${sc.host}:${sc.port}';
                        final subtitle =
                            '${sc.host}:${sc.port}'
                            '${sc.isTLS == true ? '  (TLS)' : ''}';
                        return Card(
                          margin: EdgeInsets.symmetric(
                            vertical: AppStyles.smallPadding,
                            horizontal: AppStyles.mediumPadding,
                          ),
                          child: ListTile(
                            leading: const Icon(Icons.dns),
                            title: Text(title),
                            subtitle: Text(subtitle),
                            onTap: _connectingId
                                ? null
                                : () => _connectServer(sc),
                            trailing: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                IconButton(
                                  icon: const Icon(Icons.edit),
                                  onPressed: () => _addOrEditServer(sc),
                                ),
                                IconButton(
                                  icon: const Icon(Icons.delete_outline),
                                  onPressed: () {
                                    if (sc.uniqueIdentifier != null) {
                                      ref
                                          .read(configProvider.notifier)
                                          .removeServer(sc.uniqueIdentifier!);
                                    } else {
                                      ref
                                          .read(configProvider.notifier)
                                          .setServers(
                                            config.servers
                                                .where((s) => s != sc)
                                                .toList(),
                                          );
                                    }
                                  },
                                ),
                              ],
                            ),
                          ),
                        );
                      },
                    ),
            ),
            Padding(
              padding: EdgeInsets.all(AppStyles.mediumPadding),
              child: ElevatedButton.icon(
                style: AppStyles.defaultButtonStyle,
                icon: const Icon(Icons.add),
                onPressed: () => _addOrEditServer(),
                label: Text(l10n.add),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
