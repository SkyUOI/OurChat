import 'package:flutter/material.dart';
import 'package:ourchat/const.dart';
import 'package:provider/provider.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'main.dart';
import 'ourchat/ourchat_server.dart';

class ServerSetting extends StatefulWidget {
  const ServerSetting({
    super.key,
  });

  @override
  State<ServerSetting> createState() => _ServerSettingState();
}

class _ServerSettingState extends State<ServerSetting> {
  String address = "localhost";
  int port = 7777;
  int httpPort = -1, ping = -1;
  String serverName = "", serverState = "", serverVersion = "";
  bool isOnline = false;
  OurChatServer? server;

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurchatAppState>();
    address = ourchatAppState.config!.data!["servers"][0]["host"];
    port = ourchatAppState.config!.data!["servers"][0]["port"];
    var key = GlobalKey<FormState>();
    var serverInfoLabels = Expanded(
      child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
        const Padding(
          padding: EdgeInsets.all(10.0),
          child: SizedBox(height: 100.0, width: 100.0, child: Placeholder()),
          // child: Image(image: AssetImage("assets/images/logo.png"))
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.serverAddress}: ",
            ),
            Text(
              address,
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.serverName}: ",
            ),
            Text(
              serverName,
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.port}: ",
            ),
            Text(
              port.toString(),
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.httpPort}: ",
            ),
            Text(
              (httpPort == -1 ? "" : httpPort.toString()),
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.serverStatus}: ",
            ),
            Text(
              serverState,
              style: TextStyle(
                  color: ((server != null &&
                          server!.serverStatus != null &&
                          server!.serverStatus!.value == okStatusCode)
                      ? Colors.green
                      : Colors.red)),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.serverVersion}: ",
            ),
            Text(
              serverVersion,
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              "${AppLocalizations.of(context)!.ping}: ",
            ),
            Text(
              (ping == -1 ? "" : "$ping ms"),
              style: const TextStyle(color: Colors.grey),
            )
          ],
        ),
      ]),
    );
    var serverForm = Padding(
      padding: const EdgeInsets.all(8.0),
      child: Form(
        key: key,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextFormField(
              initialValue: address,
              decoration: InputDecoration(
                  label: Text(AppLocalizations.of(context)!.serverAddress)),
              validator: (value) {
                if (value!.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }
                return null;
              },
              onSaved: (newValue) {
                setState(() {
                  address = newValue!;
                });
              },
            ),
            TextFormField(
              initialValue: port.toString(),
              decoration: InputDecoration(
                  label: Text(AppLocalizations.of(context)!.port)),
              validator: (value) {
                if (value!.isEmpty) {
                  return AppLocalizations.of(context)!.cantBeEmpty;
                }

                if (int.tryParse(value) == null ||
                    int.parse(value) > 65535 ||
                    int.parse(value) < 0) {
                  return AppLocalizations.of(context)!.invalidPort;
                }
                return null;
              },
              onSaved: (newValue) {
                setState(() {
                  port = int.parse(newValue!);
                });
              },
            ),
            Padding(
              padding: const EdgeInsets.all(10.0),
              child: ElevatedButton(
                child: Text(isOnline
                    ? AppLocalizations.of(context)!.continue_
                    : AppLocalizations.of(context)!.connect),
                onPressed: () async {
                  if (!key.currentState!.validate()) {
                    return;
                  }
                  var lastAddress = address;
                  var lastPort = port;
                  key.currentState!.save();
                  if (lastAddress == address && lastPort == port && isOnline) {
                    ourchatAppState.server = server;
                    ourchatAppState.where = authUi;
                    ourchatAppState.update();
                    return;
                  }
                  ourchatAppState.config!.data!["servers"][0]["host"] = address;
                  ourchatAppState.config!.data!["servers"][0]["port"] = port;
                  ourchatAppState.config!.saveConfig();
                  server = OurChatServer(address, port);
                  setState(() {
                    isOnline = false;
                    serverState = "";
                    httpPort = -1;
                    serverVersion = "";
                    serverName = "";
                    ping = -1;
                  });
                  var resCode = await server!.getServerInfo();
                  if (resCode == unavailableStatusCode) {
                    setState(() {
                      serverState =
                          AppLocalizations.of(context)!.serverStatusOffline;
                    });
                    return;
                  }
                  if (!context.mounted) return;
                  setState(() {
                    isOnline = true;
                    httpPort = server!.httpPort!;
                    serverState =
                        AppLocalizations.of(context)!.serverStatusOnline;
                    serverVersion =
                        "${server!.serverVersion!.major}.${server!.serverVersion!.minor}.${server!.serverVersion!.patch}";
                    serverName = server!.serverName!;
                    ping = server!.ping!;
                  });
                },
              ),
            ),
          ],
        ),
      ),
    );

    return Scaffold(body: LayoutBuilder(builder: (context, constraints) {
      if (ourchatAppState.device == mobile) {
        return Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            children: [serverInfoLabels, serverForm],
          ),
        );
      }
      return Padding(
        padding: const EdgeInsets.all(20.0),
        child: Row(
          children: [
            Flexible(flex: 1, child: serverInfoLabels),
            Flexible(flex: 2, child: serverForm)
          ],
        ),
      );
    }));
  }
}
