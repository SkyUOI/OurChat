#include <base/users.h>
#include <asio.hpp>
#include <easylogging++.h>
#include <memory>
#include <server/server.h>

using asio::ip::tcp;

namespace ourchat {
std::unordered_map<int, user_tcp_connection*> clients;
server::server(asio::io_context& io_context)
    : io_context(io_context)
    , acceptor(io_context, tcp::endpoint(tcp::v4(), port)) {
    start_accept();
}

void server::start_accept() {
    std::shared_ptr<user_tcp_connection> ptr(
        new user_tcp_connection(io_context));
    acceptor.async_accept(ptr->socket(),
        [this, ptr](auto && PH1) { handle_accept(ptr, std::forward<decltype(PH1)>(PH1)); });
}

void server::handle_accept(const std::shared_ptr<user_tcp_connection>& ptr,
    const asio::error_code& error) {
    if (!error) {
        ptr->start();
    } else {
        LOG(ERROR) << "accept socket error " << error;
    }
    start_accept();
}

server::~server() = default;
}
