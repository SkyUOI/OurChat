import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'const.dart';
import 'package:logger/logger.dart';

class OurchatConnection {
  WebSocketChannel? channel;
  Function? responseFunc;
  bool closed = true;
  String uri = "ws://localhost:7777";
  Logger logger = Logger();

  OurchatConnection(var responseFunction) {
    responseFunc = responseFunction;
  }

  void setAddress(String ip, String port) {
    uri = "ws://$ip:$port";
    logger.d("set ws uri: $uri");
  }

  void connectToServer() async {
    logger.i("try connect to WSserver");
    if (!closed) {
      logger.d("already connected\nclose old connection firstly");
      close();
    }
    try {
      channel = WebSocketChannel.connect(Uri.parse(uri));
      await channel!.ready;
      closed = false;
      channel!.stream.listen((data) {
        logger.d("receive data: $data");
        responseFunc!(jsonDecode(data));
      });
      responseFunc!({
        "code": connectServerResponse,
        "status": operationSuccessfulStatusCode
      });
    } catch (e) {
      logger.w("connect to server failed: $e");
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
      logger.d("send data: $data");
      return;
    }
    logger.w("send data failed: connection is closed");
  }

  void close() {
    logger.i("close connection");
    channel!.sink.close();
    closed = true;
  }
}
