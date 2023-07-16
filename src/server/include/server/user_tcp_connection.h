#pragma once

#include <base/login.h>
#include <base/users.h>
#include <asio.hpp>
#include <json/json.h>
#include <memory>

using asio::ip::tcp;

namespace ourchat {
/**
 * @brief 用户tcp连接
 */
class user_tcp_connection
    : public std::enable_shared_from_this<user_tcp_connection> {
public:
    user_tcp_connection(asio::io_context& io_context);

    ~user_tcp_connection();

    tcp::socket& socket();

    void start();

private:
    void read_res(
        const asio::error_code& error, size_t bytes_transferred);
    tcp::socket socket_;

    /**
     * @brief 尝试登录
     */
    void trylogin(const Json::Value& value);

    /**
     * @brief 发送文本
     * @param text 文本
     * @param group 聊天信息
     */
    void send_text(const Json::Value& json);

    /**
     * @brief 尝试注册
     */
    void tryregister(const Json::Value& value);

    void handle_write(
        const asio::error_code& error, size_t bytes_transferred);

    char json_tmp[1024];

    int user_id;
};
}
