#include <asio.hpp>
#include <iostream>
#include <json/json.h>
#include <server/server_def.h>

using asio::ip::tcp;
char readbuf[1024];

int main() {
    asio::io_context io_context;
    tcp::resolver resolver(io_context);
    tcp::resolver::results_type endpoints
        = resolver.resolve("127.0.0.1", "54088");
    tcp::socket socket(io_context);
    asio::connect(socket, endpoints);
    asio::error_code ignored_error;
    asio::write(socket,
        asio::buffer("{"
                     "  \"code\": 6,"
                     "  \"time\": 1661389837,"
                     "  \"data\": {"
                     "    \"email\": \"limuyang2020@163.com\","
                     "    \"password\": \"123456\""
                     "  }"
                     "}"),
        ignored_error);
    size_t len = socket.read_some(asio::buffer(readbuf), ignored_error);
    std::cout << readbuf;
    return 0;
}
