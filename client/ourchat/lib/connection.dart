import 'dart:convert';
import 'package:web_socket_channel/io.dart';
import 'const.dart';
import 'dart:io';

class OurchatConnection {
  WebSocket? connection;
  IOWebSocketChannel? channel;
  Function? responseFunc;
  bool closed = true;
  String uri = "ws://localhost:7777";

  OurchatConnection(var responseFunction) {
    responseFunc = responseFunction;
  }

  void setAddress(String ip, String port) {
    uri = "ws://$ip:$port";
  }

  void connectToServer() async {
    if (!closed) {
      close();
    }
    try {
      connection = await WebSocket.connect(uri);
      channel = IOWebSocketChannel(connection!);
      channel!.ready.then((_) {
        closed = false;
        channel!.stream.listen((data) {
          responseFunc!(jsonDecode(data));
        });
        responseFunc!({
          "code": connectServerResponse,
          "status": operationSuccessfulStatusCode
        });
      });
    } catch (e) {
      responseFunc!({
        "code": connectServerResponse,
        "status": unknownErrorStatusCode,
        "msg": e.toString()
      });
    }
  }

  void send(var data) {
    if (!closed) {
      channel!.sink.add(jsonEncode(data));
    }
  }

  void close() {
    channel!.sink.close();
    closed = true;
  }
}
