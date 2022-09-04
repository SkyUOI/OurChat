#include <boost/asio.hpp>
#include <iostream>
#include <json/json.h>
#include <server/server_def.h>

using boost::asio::ip::tcp;
char readbuf[1024];

int main() {
    boost::asio::io_context io_context;
    tcp::resolver resolver(io_context);
    tcp::resolver::results_type endpoints
        = resolver.resolve("127.0.0.1", "54088");
    tcp::socket socket(io_context);
    boost::asio::connect(socket, endpoints);
    boost::system::error_code ignored_error;
    boost::asio::write(socket,
        boost::asio::buffer("{"
                            "  \"code\": 6,"
                            "  \"time\": 1661389837,"
                            "  \"data\": {"
                            "    \"email\": \"limuyang2020@163.com\","
                            "    \"password\": \"123456\""
                            "  }"
                            "}"),
        ignored_error);
    size_t len = socket.read_some(boost::asio::buffer(readbuf), ignored_error);
    std::cout << readbuf;
    return 0;
}
