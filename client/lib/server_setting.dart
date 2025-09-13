import 'package:flutter/material.dart';
import 'package:ourchat/core/const.dart';
import 'package:provider/provider.dart';
import 'package:ourchat/l10n/app_localizations.dart';
import 'package:ourchat/main.dart';
import 'package:ourchat/auth.dart';
import 'package:ourchat/core/server.dart';

class ServerSetting extends StatefulWidget {
  const ServerSetting({super.key});

  @override
  State<ServerSetting> createState() => _ServerSettingState();
}

class _ServerSettingState extends State<ServerSetting> {
  String address = "localhost";
  int port = 7777;
  int httpPort = -1, ping = -1;
  String serverName = "", serverState = "", serverVersion = "";
  bool isOnline = false, isConnecting = false, inited = false;
  bool? isTLS;
  late OurChatServer server;
  Color serverStatusColor = Colors.grey;

  @override
  Widget build(BuildContext context) {
    var ourchatAppState = context.watch<OurChatAppState>();
    var l10n = AppLocalizations.of(context)!;
    // 从配置中读取地址和端口
    if (!inited) {
      address = ourchatAppState.config["servers"][0]["host"];
      port = ourchatAppState.config["servers"][0]["port"];
      inited = true;
    }
    var key = GlobalKey<FormState>();
    var serverInfoLabels = Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Padding(
          padding: EdgeInsets.all(10.0),
          child: SizedBox(height: 100.0, width: 100.0, child: Placeholder()),
          // child: Image(image: AssetImage("assets/images/logo.png"))
        ),
        Row(
          // 展示服务端ip
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.serverAddress}: "),
            Text(address, style: const TextStyle(color: Colors.grey)),
          ],
        ),
        Row(
          // 展示服务端名称
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.serverName}: "),
            Text(serverName, style: const TextStyle(color: Colors.grey)),
          ],
        ),
        Row(
          // 展示服务端端口
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.port}: "),
            Text(port.toString(), style: const TextStyle(color: Colors.grey)),
          ],
        ),
        Row(
          // 展示服务端http端口
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.httpPort}: "),
            Text(
              (httpPort == -1 ? "" : httpPort.toString()),
              style: const TextStyle(color: Colors.grey),
            ),
          ],
        ),
        Row(
          // 展示服务端状态
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.serverStatus}: "),
            Text(serverState, style: TextStyle(color: serverStatusColor)),
          ],
        ),
        Row(
          // 展示服务端版本
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.serverVersion}: "),
            Text(serverVersion, style: const TextStyle(color: Colors.grey)),
          ],
        ),
        Row(
          // 展示连接延迟
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.ping}: "),
            Text(
              (ping == -1 ? "" : "$ping ms"),
              style: const TextStyle(color: Colors.grey),
            ),
          ],
        ),
        Row(
          // 展示是否支持tls
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text("${l10n.tlsEncryption} "),
            isTLS == null
                ? Text("")
                : (isTLS!
                    ? Text(
                        l10n.tlsEnabled,
                        style: const TextStyle(color: Colors.green),
                      )
                    : Text(
                        l10n.tlsDisabled,
                        style: const TextStyle(color: Colors.red),
                      ))
          ],
        ),
      ],
    );
    var serverForm = Padding(
      padding: const EdgeInsets.all(8.0),
      child: Form(
        key: key,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextFormField(
              // 地址输入框
              initialValue: address,
              decoration: InputDecoration(
                label: Text(l10n.serverAddress),
              ),
              validator: (value) {
                if (value!.isEmpty) {
                  return l10n.cantBeEmpty;
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
              // 端口输入框
              initialValue: port.toString(),
              decoration: InputDecoration(
                label: Text(l10n.port),
              ),
              validator: (value) {
                if (value!.isEmpty) {
                  return l10n.cantBeEmpty;
                }

                if (int.tryParse(value) == null ||
                    int.parse(value) > 65535 ||
                    int.parse(value) < 0) {
                  return l10n.invalid(l10n.port);
                }
                return null;
              },
              onSaved: (newValue) {
                setState(() {
                  port = int.parse(newValue!);
                });
              },
            ),
            if (!isConnecting) // 没有连接进程
              Padding(
                padding: const EdgeInsets.all(10.0),
                child: ElevatedButton(
                  child: Text(
                    // 如果服务端在线(尝试连接成功)，则显示"继续"
                    isOnline ? l10n.continue_ : l10n.connect,
                  ),
                  onPressed: () async {
                    if (!key.currentState!.validate()) {
                      // 检查服务端信息是否合法
                      return;
                    }
                    setState(() {
                      isConnecting = true;
                    });
                    var prevAddress = address;
                    var prevPort = port;
                    key.currentState!.save();
                    if (prevAddress == address &&
                        prevPort == port &&
                        isOnline) {
                      // 进入Auth界面
                      ourchatAppState.server = server;
                      ourchatAppState.update();
                      Navigator.pop(context);
                      Navigator.push(context,
                          MaterialPageRoute(builder: (context) {
                        return const Scaffold(
                          body: Auth(),
                        );
                      }));
                      return;
                    }
                    // 连接新的服务端地址
                    isTLS = await OurChatServer.tlsEnabled(address, port);
                    server = OurChatServer(address, port, isTLS!);
                    setState(() {
                      isOnline = false;
                      serverState = "";
                      httpPort = -1;
                      serverVersion = "";
                      serverName = "";
                      ping = -1;
                      serverStatusColor = Colors.grey;
                    });
                    var resCode = unavailableStatusCode;
                    resCode = await server.getServerInfo();
                    setState(() {
                      isConnecting = false;
                    });
                    if (resCode == unavailableStatusCode ||
                        resCode == unknownStatusCode) {
                      // 连接失败
                      setState(() {
                        serverState = l10n.serverStatusOffline;
                        serverStatusColor = Colors.red;
                      });
                      return;
                    }
                    // 连接成功
                    if (!context.mounted) return;
                    ourchatAppState.config["servers"][0]["host"] = address;
                    ourchatAppState.config["servers"][0]["port"] = port;
                    ourchatAppState.config.saveConfig();
                    // 保存服务器地址
                    setState(() {
                      isOnline = true;
                      httpPort = server.httpPort!;
                      switch (server.serverStatus!.value) {
                        case okStatusCode:
                          serverState = l10n.serverStatusOnline;
                          serverStatusColor = Colors.green;
                          break;
                        case internalStatusCode:
                          serverState = l10n.serverError;
                          serverStatusColor = Colors.red;
                          break;
                        case unavailableStatusCode:
                          serverState = AppLocalizations.of(
                            context,
                          )!
                              .serverStatusUnderMaintenance;
                          serverStatusColor = Colors.orange;
                          break;
                        default:
                          serverState = l10n.serverStatusUnknown;
                          serverStatusColor = Colors.grey;
                          break;
                      }
                      serverVersion =
                          "${server.serverVersion!.major}.${server.serverVersion!.minor}.${server.serverVersion!.patch}";
                      serverName = server.serverName!;
                      ping = server.ping!;
                    });
                  },
                ),
              ),
            if (isConnecting) // 连接中
              Padding(
                padding: EdgeInsets.all(10.0),
                child: CircularProgressIndicator(
                  color: Theme.of(context).primaryColor,
                ),
              )
          ],
        ),
      ),
    );

    return Scaffold(
      body: LayoutBuilder(
        builder: (context, constraints) {
          if (ourchatAppState.device == mobile) {
            // 移动端，纵向展示
            return Padding(
              padding: const EdgeInsets.all(20.0),
              child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [serverInfoLabels, serverForm]),
            );
          }
          return Padding(
            // 桌面端，横向展示
            padding: const EdgeInsets.all(20.0),
            child: Row(
              children: [
                Flexible(flex: 1, child: serverInfoLabels),
                Flexible(flex: 2, child: serverForm),
              ],
            ),
          );
        },
      ),
    );
  }
}
