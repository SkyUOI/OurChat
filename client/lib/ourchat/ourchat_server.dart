import 'package:grpc/grpc.dart';
import 'package:ourchat/const.dart';
import 'package:ourchat/service/basic/server/v1/server.pb.dart';
import 'package:ourchat/service/basic/v1/basic.pbgrpc.dart';

class OurchatInterceptor implements ClientInterceptor {
  String token = "";
  void setToken(String t) {
    token = t;
  }

  @override
  ResponseFuture<R> interceptUnary<Q, R>(ClientMethod<Q, R> method, Q request,
      CallOptions options, ClientUnaryInvoker<Q, R> invoker) {
    var newOptions = options.mergedWith(
      CallOptions(metadata: {'token': token}),
    );
    return invoker(method, request, newOptions);
  }

  @override
  ResponseStream<R> interceptStreaming<Q, R>(
      ClientMethod<Q, R> method,
      Stream<Q> requests,
      CallOptions options,
      ClientStreamingInvoker<Q, R> invoker) {
    var newOptions = options.mergedWith(
      CallOptions(metadata: {'token': token}),
    );
    return invoker(method, requests, newOptions);
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
  OurchatInterceptor? interceptor;

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
