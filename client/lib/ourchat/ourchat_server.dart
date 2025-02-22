import 'package:grpc/grpc.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/service/basic/server/v1/server.pb.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

class OurChatInterceptor extends ClientInterceptor {
  String? token;
  void setToken(String t) {
    token = t;
  }

  @override
  ResponseFuture<R> interceptUnary<Q, R>(
    ClientMethod<Q, R> method,
    Q request,
    CallOptions options,
    invoker,
  ) {
    var newOptions = CallOptions.from([options])
      ..metadata.putIfAbsent('token', () => token!);
    return invoker(method, request, newOptions);
  }
}

class OurChatServer {
  String host;
  String? uniqueIdentifier, serverName;
  int port;
  int? httpPort, ping;
  RunningStatus? serverStatus;
  ClientChannel? channel;
  ServerVersion? serverVersion;
  OurChatInterceptor? interceptor;

  OurChatServer(this.host, this.port) {
    channel = ClientChannel(
      host,
      port: port,
      options: const ChannelOptions(credentials: ChannelCredentials.insecure()),
    );
  }

  Future getServerInfo() async {
    BasicServiceClient stub = BasicServiceClient(channel!);
    try {
      int beginTime = DateTime.now().millisecondsSinceEpoch;
      var res = await stub.getServerInfo(GetServerInfoRequest());
      int endTime = DateTime.now().millisecondsSinceEpoch;
      ping = endTime - beginTime;
      serverStatus = res.status;
      httpPort = res.httpPort;
      uniqueIdentifier = res.uniqueIdentifier;
      serverVersion = res.serverVersion;
      serverName = res.serverName;
      return okStatusCode;
    } on GrpcError catch (e) {
      return e.code;
    } catch (e) {
      return unknownStatusCode;
    }
  }
}
