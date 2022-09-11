#include <base/users.h>
#include <boost/asio.hpp>
#include <boost/bind/bind.hpp>
#include <easylogging++.h>
#include <memory>
#include <server/server.h>

using boost::asio::ip::tcp;

namespace ourchat {
std::unordered_map<int, user_tcp_connection*> clients;
server::server(boost::asio::io_context& io_context)
    : io_context(io_context)
    , acceptor(io_context, tcp::endpoint(tcp::v4(), port)) {
    start_accept();
}

void server::start_accept() {
    std::shared_ptr<user_tcp_connection> ptr(
        new user_tcp_connection(io_context));
    acceptor.async_accept(ptr->socket(),
        boost::bind(&server::handle_accept, this, ptr,
            boost::asio::placeholders::error));
}

void server::handle_accept(std::shared_ptr<user_tcp_connection> ptr,
    const boost::system::error_code& error) {
    if (!error) {
        ptr->start();
    } else {
        LOG(ERROR) << "accept socket error " << error;
    }
    start_accept();
}

server::~server() {
}
}
