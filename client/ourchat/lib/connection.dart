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

  void setAddress(String ip, int port) {
    uri = "ws://$ip:$port";
  }

  void connectToServer() {
    if (!closed) {
      close();
    }
    channel = WebSocketChannel.connect(Uri.parse(uri));
    channel!.ready.then((_) => already());
  }

  void already() {
    channel!.stream.listen((event) {
      var data = jsonDecode(event);
      responseFunc!(data);
    });
    closed = false;
    send({"code": serverStatusMsgCode});
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
