#include <server/server.h>
#include <boost/asio.hpp>
#include <jsoncpp/json/json.h>
#include <boost/array.hpp>
#include <sstream>
#include <base/login.h>
#include <base/users.h>
#include <iostream>
#include <easyloggingpp/easylogging++.h>

using boost::asio::ip::tcp;

namespace ourchat {
    char readbuf[1024];
    server::server():acceptor(io_context, tcp::endpoint(tcp::v4(), port)) {
        Json::Reader reader;
        for(;;) {
            // 客户端套接字
            tcp::socket socket(io_context);
            acceptor.accept(socket);
            boost::system::error_code error;
            // 读取到缓冲区
            size_t len = socket.read_some(boost::asio::buffer(readbuf), error);
            // 解析json数据
            Json::Value root;
            reader.parse(readbuf, root);
            auto receive_code = client_code(root["code"].asInt());
            switch(receive_code) {
                case client_code::LOGIN:{
                    // 进入登录
                    trylogin(socket);
                    break;
                }
                case client_code::TEXT:{
                    // 发送文本信息
                    send_text(readbuf, root["data"]["cid"].asUInt());
                    break;
                }
                default:{
                    NOREACH("code %d is not defined.", receive_code);
                }
            }
        }
    }

    static std::string make_login_json(const std::string& state_code) {
        std::string s = "{\"code\":7, \"data\":{\"state\":";
        s += state_code;
        s += "}}";
        return s;
    }

    void server::trylogin(tcp::socket& socket) {
        boost::system::error_code error;
        database::login_state return_code = database::login();
        switch(return_code) {
            case database::login_state::SUCCESS:{
                boost::asio::write(socket, boost::asio::buffer(make_login_json("0")), error);
                // 正常登录
                break;
            }
            case database::login_state::PASSWORDINCORRECT:
            case database::login_state::ACCOUNTNOTFOUND:{
                boost::asio::write(socket, boost::asio::buffer(make_login_json("1")), error);
                // 账号未定义或密码错误
                break;
            }
            default:{
                NOREACH("code %d is not defined.", return_code);
            }
        }
    }

    void server::send_text(const std::string& json, group_id_t group) {
        // 首先根据group_id获取群聊的所有人数，然后按照OCID发送给具体的socket，未存在socket链接则储存信息
        boost::system::error_code error;
        for(const auto&i:database::get_members_by_group(group)) {
            if(clients.find(i) != clients.end()) {
                // 存在socket链接,直接发送
                clients[i];
                // boost::asio::write(clients[i], boost::asio::buffer(json), error);
            } else {
                // 不存在socket链接，保存数据到数据库
                database::save_chat_msg(i, json);
            }
        }
    }

    server::~server() {
    }
}
