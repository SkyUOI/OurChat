import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'const.dart';

class OurchatConnection {
  WebSocketChannel? channel;
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
      channel = WebSocketChannel.connect(Uri.parse(uri));
      await channel!.ready;
      closed = false;
      channel!.stream.listen((data) {
        responseFunc!(jsonDecode(data));
      });
      responseFunc!({
        "code": connectServerResponse,
        "status": operationSuccessfulStatusCode
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
