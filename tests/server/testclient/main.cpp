#include "base/server_def.h"
#include "boost/array.hpp"
#include "boost/asio.hpp"
#include "json/json.h"
#include <cstdio>
#include <filesystem>
#include <iostream>
#include <thread>
#include <windows.h>

using boost::asio::ip::tcp;
char readbuf[1024];

int main() {
    //     Json::Reader parse;
    //     Json::Value root;
    //     parse.parse("{\"p\":\"o\"}", root);
    //     std::cout << root["p"].asString();
    boost::asio::io_context io_context;
    tcp::resolver resolver(io_context);
    tcp::resolver::results_type endpoints
        = resolver.resolve("127.0.0.1", "54088");
    tcp::socket socket(io_context);
    boost::asio::connect(socket, endpoints);
    boost::system::error_code ignored_error;
    boost::asio::write(socket,
        boost::asio::buffer("{\"code\" : 6,\"data\" : {\"ocId\" : "
                            "\"aaa\",\"password\" : \"123456\"}}"),
        ignored_error);
    size_t len = socket.read_some(boost::asio::buffer(readbuf), ignored_error);
    std::cout << readbuf;
    return 0;
}
