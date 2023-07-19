/**
 * @brief server class
 */
#pragma once
#include <asio.hpp>
#include <json/json.h>
#include <memory>
#include <server/server_def.h>
#include <server/user_tcp_connection.h>
#include <unordered_map>

using asio::ip::tcp;

namespace ourchat {
// 储存oc号对应的客户端
extern std::unordered_map<int, user_tcp_connection*> clients;
/**
 * @brief 一个异步服务器
 */
class server {
public:
    server(asio::io_context& io_context);

    ~server();

private:
    /**
     * @brief 开始接收socket
     */
    void start_accept();

    /**
     * @brief accept到socket的回调函数
     */
    void handle_accept(const std::shared_ptr<user_tcp_connection>& ptr,
        const asio::error_code& error);

    asio::io_context& io_context;

    tcp::acceptor acceptor;
};
}
