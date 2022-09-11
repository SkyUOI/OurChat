#pragma once

#include <base/login.h>
#include <base/users.h>
#include <boost/asio.hpp>
#include <boost/bind/bind.hpp>
#include <json/json.h>
#include <memory>

using boost::asio::ip::tcp;

namespace ourchat {
/**
 * @brief 用户tcp连接
 */
class user_tcp_connection
    : public std::enable_shared_from_this<user_tcp_connection> {
public:
    user_tcp_connection(boost::asio::io_context& io_context);

    ~user_tcp_connection();

    tcp::socket& socket();

    void start();

private:
    void read_res(
        const boost::system::error_code& error, size_t bytes_transferred);
    tcp::socket socket_;

    /**
     * @brief 尝试登录
     */
    void trylogin(const Json::Value& value);

    /**
     * @brief 发送文本
     * @param text 文本
     * @param group 聊天号
     */
    void send_text(group_id_t group);

    /**
     * @brief 尝试注册
     */
    void tryregister(const Json::Value& value);

    void handle_write(
        const boost::system::error_code& error, size_t bytes_transferred);

    char json_tmp[1024];

    int user_id;
};
}
