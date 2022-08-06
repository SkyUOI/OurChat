#include <cstdio>
#include <boost/asio.hpp>
#include <boost/array.hpp>
#include <iostream>

using boost::asio::ip::tcp;

int main(int argc, char *argv[])
{
    puts("OurChat Server starting...");
    boost::asio::io_context io;
    tcp::resolver resolver(io);
    tcp::resolver::results_type endpoints = resolver.resolve(argv[1], argv[2]);
    tcp::socket socket(io);
    boost::asio::connect(socket, endpoints);
    try{
        for(;;) {
            boost::array<char,128> buf;
            boost::system::error_code error;
            size_t len = socket.read_some(boost::asio::buffer(buf), error);
            if(error == boost::asio::error::eof) {
                break;
            }
            std::cout.write(buf.data(), len);
        }
    } catch(std::exception &e) {
        std::cerr << e.what();
    }
    
    return 0;
}
