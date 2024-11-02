import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:ourchat/connection.dart';
// import 'package:mockito/mockito.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

// import 'connection_test.mocks.dart';

@GenerateMocks([WebSocketChannel])
void main() {
  // final mockWebSocketChannel = MockWebSocketChannel();
  group("Connection test", () {
    test("test default address", () {
      OurchatConnection connection = OurchatConnection(null);
      expect(connection.uri, "ws://localhost:7777");
    });

    test("test set address", () {
      OurchatConnection connection = OurchatConnection(null);
      connection.setAddress("test.com", "7778");
      expect(connection.uri, "ws://test.com:7778");
    });

    // test("test connect to server", () {
    //   void responseFunc(var data) {}
    // });
  });
}
