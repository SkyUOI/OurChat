#include <base/login.h>
#include <base/users.h>
#include <boost/asio.hpp>
#include <boost/bind/bind.hpp>
#include <easylogging++.h>
#include <json/json.h>
#include <memory>
#include <server/server.h>
#include <server/user_tcp_connection.h>

#define write_handle                                                           \
    (boost::bind(&user_tcp_connection::handle_write, shared_from_this(),       \
        boost::asio::placeholders::error,                                      \
        boost::asio::placeholders::bytes_transferred))

namespace ourchat {
user_tcp_connection::user_tcp_connection(boost::asio::io_context& io_context)
    : socket_(io_context) {
}

tcp::socket& user_tcp_connection::socket() {
    return socket_;
}

void user_tcp_connection::read_res(
    const boost::system::error_code& error, size_t bytes_transferred) {
    if (error == boost::asio::error::eof) {
        // 连接已结束
        return;
    }
    if (!error) {
        Json::Reader reader;
        // 正常
        // 解析json数据
        Json::Value root;
        reader.parse(json_tmp, root);
        auto receive_code = client_code(root["code"].asInt());
        switch (receive_code) {
        case client_code::LOGIN: {
            // 进入登录
            trylogin(root);
            break;
        }
        case client_code::TEXT: {
            // 发送文本信息
            send_text(root["data"]["cid"].asUInt());
            break;
        }
        case client_code::REGISTER: {
            tryregister(root);
            break;
        }
        default: {
            LOG(ERROR) << "client code " << int(receive_code)
                       << " is not defined.";
        }
        }
    } else {
        LOG(ERROR) << "boost.asio error listen to user " << error;
    }
}

void user_tcp_connection::start() {
    // 读取到缓冲区
    socket_.async_read_some(boost::asio::buffer(json_tmp),
        boost::bind(&user_tcp_connection::read_res, shared_from_this(),
            boost::asio::placeholders::error,
            boost::asio::placeholders::bytes_transferred));
}

void user_tcp_connection::handle_write(
    const boost::system::error_code& error, size_t bytes_transferred) {
}

void user_tcp_connection::trylogin(const Json::Value& value) {
    database::login_return return_code;
    const std::string& passwd = value["data"]["password"].asString();
    if (value["data"].isMember("ocId")) {
        return_code
            = database::login<false>(value["data"]["ocId"].asString(), passwd);
    } else {
        return_code
            = database::login<true>(value["data"]["email"].asString(), passwd);
    }
    switch (return_code.state) {
    case database::login_state::SUCCESS: {
        // 正常登录
        sprintf(json_tmp, R"({"code":7, "data":{"state":0, "id":%d}})",
            return_code.id);
        boost::asio::async_write(
            socket_, boost::asio::buffer(json_tmp), write_handle);
        // 保存套接字
        user_id = return_code.id;
        clients[user_id] = this;
        break;
    }
    case database::login_state::PASSWORDINCORRECT:
    case database::login_state::ACCOUNTNOTFOUND: {
        // 账号未定义或密码错误
        boost::asio::async_write(socket_,
            boost::asio::buffer("{"
                                "\"code\":7,"
                                "\"data\":{"
                                "\"state\":1"
                                "}"
                                "}"),
            write_handle);
        break;
    }
    default: {
        LOG(ERROR) << "login code " << int(return_code.state)
                   << " is not defined.";
        boost::asio::async_write(socket_,
            boost::asio::buffer("{"
                                "\"code\":7, "
                                "\"data\":{"
                                "\"state\":2"
                                "}"
                                "}"),
            write_handle);
    }
    }
}

void user_tcp_connection::send_text(group_id_t group) {
    // 首先根据group_id获取群聊的所有人数，然后按照OCID发送给具体的socket，未存在socket链接则储存信息
    boost::system::error_code error;
    MYSQL_RES* group_members = database::get_members_by_group(group);
    if (group_members == nullptr) {
        return;
    }
    bool should_be_saved = false;
    int msg_id;
    for (MYSQL_ROW i = mysql_fetch_row(group_members); i != nullptr;
         i = mysql_fetch_row(group_members)) {
        int iint = atoi(i[0]);
        if (clients.find(iint) != clients.end()) {
            // 存在socket链接
            boost::asio::async_write(clients[iint]->socket(),
                boost::asio::buffer(json_tmp), write_handle);
            // 判断是否成功发送
            if (error == boost::asio::error::eof) {
                // 连接已结束,保存数据到数据库,等待下一次发送
                clients.erase(iint);
                goto failed;
            }
        } else {
            // 不存在socket链接，保存数据到数据库,等待下一次发送
            goto failed;
        }
    failed:
        if (!should_be_saved) {
            // 先保存到数据库中
            should_be_saved = true;
            msg_id = database::saved_msg(json_tmp);
        }
        database::save_chat_msg(iint, msg_id);
    }
}

void user_tcp_connection::tryregister(const Json::Value& value) {
    // 通过json解析出数据
    database::user_for_register user;
    user.passwd = value["data"]["password"].asString();
    user.name = value["data"]["name"].asString();
    user.date = value["time"].asInt();
    user.email = value["data"]["email"].asString();
    database::register_return returncode = database::register_(user);
    switch (returncode.state) {
    case database::register_state::SUCCESS: {
        // 注册成功
        sprintf(json_tmp,
            "{"
            "\"code\": 5,"
            "\"data\": {"
            "\"state\": 0,"
            "\"ocId\": \"%s\","
            "\"id\":%d"
            "}"
            "}",
            returncode.ocid.c_str(), returncode.id);
        boost::asio::async_write(
            socket_, boost::asio::buffer(json_tmp), write_handle);
        break;
    }
    case database::register_state::EMAIL_DUP: {
        sprintf(json_tmp,
            "{"
            "\"code\": 5,"
            "\"data\": {"
            "\"state\": 2"
            "}"
            "}");
        boost::asio::async_write(
            socket_, boost::asio::buffer(json_tmp), write_handle);
        break;
    }
    default: {
        sprintf(json_tmp,
            "{"
            "\"code\": 5,"
            "\"data\": {"
            "\"state\": 1"
            "}"
            "}");
        boost::asio::async_write(
            socket_, boost::asio::buffer(json_tmp), write_handle);
        LOG(ERROR) << "register code " << (int)returncode.state
                   << " is not defined.";
    }
    }
}

user_tcp_connection::~user_tcp_connection() {
    socket_.close();
    clients.erase(user_id);
}
}
