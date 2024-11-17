import 'package:grpc/grpc.dart';
import 'package:ourchat/google/protobuf/empty.pb.dart';
import 'package:ourchat/message/service.pbgrpc.dart';
import 'package:protobuf/protobuf.dart';
import 'const.dart';
import 'package:logger/logger.dart';
import 'package:ourchat/message/service.pb.dart';

class OurchatConnection {
  ClientChannel? channel;
  Function? responseFunc;
  bool closed = true;
  String host = "localhost";
  int port = 7777;
  Logger logger = Logger();

  OurchatConnection(var responseFunction) {
    responseFunc = responseFunction;
  }

  void setAddress(String host_, int port_) {
    host = host_;
    port = port_;
  }

  void connectToServer() async {
    channel = ClientChannel(host, port: port);
    final stub = BasicServiceClient(channel!);
    final res = await stub.get_server_info(Empty());
    print(res.status.value);
  }
}
