/**
 * @brief server class
 */
#pragma once
#include <server/server_def.h>
#include <boost/asio.hpp>
#include <json/json.h>
#include <unordered_map>

using boost::asio::ip::tcp;

namespace ourchat {
class server {
public:
    server();

    ~server();

private:
    /**
     * @brief 尝试登录
     */
    void trylogin(tcp::socket& socket, Json::Value value);

    /**
     * @brief 发送文本
     * @param text 文本
     * @param group 聊天号
     */
    void send_text(const std::string& json, group_id_t group);

    /**
     * @brief 尝试注册
     */
    void tryregister(tcp::socket& socket, Json::Value value);

    boost::asio::io_context io_context;

    tcp::acceptor acceptor;

    // 储存oc号对应的客户端
    std::unordered_map<ocid_t, tcp::socket*> clients;
};
}
