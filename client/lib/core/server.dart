import 'package:grpc/grpc.dart';
import 'package:ourchat/core/const.dart';
import 'package:ourchat/core/log.dart';
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
      CallOptions(metadata: {'authorization': "Bearer $token"}),
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
      CallOptions(metadata: {'authorization': "Bearer $token"}),
    );
    return invoker(method, requests, newOptions);
  }
}

class OurchatServer {
  String host;
  String? uniqueIdentifier, serverName;
  int port;
  int? httpPort, ping;
  RunningStatus? serverStatus;
  ClientChannel? channel;
  ServerVersion? serverVersion;
  OurchatInterceptor? interceptor;

  OurchatServer(this.host, this.port, bool ssl, int pingIntervalSeconds) {
    // try ssl/tls connection
    if (!ssl) {
      logger.w("Switch to insecure connection");
    }
    ChannelCredentials credentials = ChannelCredentials.insecure();
    if (ssl) {
      credentials = ChannelCredentials.secure();
    }
    channel = ClientChannel(host,
        port: port,
        options: ChannelOptions(
            credentials: credentials,
            keepAlive: ClientKeepAliveOptions(
                pingInterval: Duration(seconds: pingIntervalSeconds))));
  }

  static Future<bool> tlsEnabled(String host, int port) async {
    try {
      var channel = ClientChannel(
        host,
        port: port,
        options: const ChannelOptions(credentials: ChannelCredentials.secure()),
      );
      var stub = BasicServiceClient(channel);
      await stub.ping(PingRequest());
      return true;
    } on GrpcError catch (e) {
      if (e.code == unavailableStatusCode &&
          e.message!.contains("HandshakeException")) {}
      return false;
    }
  }

  Future getServerInfo() async {
    BasicServiceClient stub = BasicServiceClient(channel!);
    try {
      int beginTime = DateTime.now().millisecondsSinceEpoch;
      var _ = await stub.ping(PingRequest());
      int endTime = DateTime.now().millisecondsSinceEpoch;
      ping = endTime - beginTime;

      var res = await stub.getServerInfo(GetServerInfoRequest());
      serverStatus = res.status;
      httpPort = res.httpPort;
      uniqueIdentifier = res.uniqueIdentifier;
      serverVersion = res.serverVersion;
      serverName = res.serverName;
      return okStatusCode;
    } on GrpcError catch (e) {
      logger.e("Failed to get server info: ${e.message}");
      return e.code;
    } catch (e) {
      logger.e("Failed to get server info: $e");
      return unknownStatusCode;
    }
  }
}
