import 'package:grpc/grpc.dart';
import 'package:ourchat/google/protobuf/empty.pb.dart';
import 'package:protobuf/protobuf.dart';
import 'const.dart';
import 'package:logger/logger.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

class OurchatConnection {
  ClientChannel? channel;
  String host = "localhost";
  int port = 7777;
  Logger logger = Logger();

  void setAddress(String host_, int port_) {
    host = host_;
    port = port_;
  }

  Future connectToServer() async {
    channel = ClientChannel(
      host,
      port: port,
      options: const ChannelOptions(credentials: ChannelCredentials.insecure()),
    );
    final stub = BasicServiceClient(channel!);
    try {
      final res = await stub.getServerInfo(GetServerInfoRequest());
      return {"status": res.status.value, "msg": res.status.name};
    } catch (e) {
      return {
        "status": cannotConnectServer,
        "msg": e.toString()
      }; // cannot connect
    }
  }
}
