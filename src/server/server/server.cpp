#include <base/login.h>
#include <base/users.h>
#include <boost/asio.hpp>
#include <easylogging++.h>
#include <json/json.h>
#include <server/server.h>

using boost::asio::ip::tcp;

namespace ourchat {
char readbuf[1024];
server::server()
    : acceptor(io_context, tcp::endpoint(tcp::v4(), port)) {
    Json::Reader reader;
    while (true) {
        // 客户端套接字
        auto* sockptr = new tcp::socket(io_context);
        acceptor.accept(*sockptr);
        boost::system::error_code error;
        // 读取到缓冲区
        size_t len = sockptr->read_some(boost::asio::buffer(readbuf), error);
        // 解析json数据
        Json::Value root;
        reader.parse(readbuf, root);
        auto receive_code = client_code(root["code"].asInt());
        switch (receive_code) {
        case client_code::LOGIN: {
            // 进入登录
            trylogin(*sockptr, root);
            break;
        }
        case client_code::TEXT: {
            // 发送文本信息
            send_text(readbuf, root["data"]["cid"].asUInt());
            break;
        }
        case client_code::REGISTER: {
            tryregister(*sockptr, root);
            break;
        }
        default: {
            LOG(ERROR) << "client code " << int(receive_code)
                       << " is not defined.";
        }
        }
    }
}

char json_tmp[1024];

void server::trylogin(tcp::socket& socket, const Json::Value& value) {
    boost::system::error_code error;
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
        boost::asio::write(socket, boost::asio::buffer(json_tmp), error);
        // 保存套接字
        clients[return_code.id] = &socket;
        break;
    }
    case database::login_state::PASSWORDINCORRECT:
    case database::login_state::ACCOUNTNOTFOUND: {
        // 账号未定义或密码错误
        boost::asio::write(socket,
            boost::asio::buffer("{"
                                "\"code\":7, "
                                "\"data\":{"
                                "\"state\":1"
                                "}"
                                "}"),
            error);
        break;
    }
    default: {
        LOG(ERROR) << "login code " << int(return_code.state)
                   << " is not defined.";
        boost::asio::write(socket,
            boost::asio::buffer("{"
                                "\"code\":7, "
                                "\"data\":{"
                                "\"state\":2"
                                "}"
                                "}"),
            error);
    }
    }
}

void server::send_text(const std::string& json, group_id_t group) {
    // 首先根据group_id获取群聊的所有人数，然后按照OCID发送给具体的socket，未存在socket链接则储存信息
    boost::system::error_code error;
    MYSQL_RES* group_members = database::get_members_by_group(group);
    if (group_members == nullptr) {
        return;
    }
    for (MYSQL_ROW i = mysql_fetch_row(group_members); i != nullptr;
         i = mysql_fetch_row(group_members)) {
        int iint = atoi(i[0]);
        if (clients.find(iint) != clients.end()) {
            // 存在socket链接
            boost::asio::write(
                *(clients[iint]), boost::asio::buffer(json), error);
            // 判断是否成功发送
            if (error == boost::asio::error::eof) {
                // 连接已结束保存数据到数据库,等待下一次发送
                database::save_chat_msg(iint, json);
                delete clients[iint];
                clients.erase(iint);
            }
        } else {
            // 不存在socket链接，保存数据到数据库,等待下一次发送
            database::save_chat_msg(iint, json);
        }
    }
}

void server::tryregister(tcp::socket& socket, const Json::Value& value) {
    // 通过json解析出数据
    database::user_for_register user;
    user.passwd = value["data"]["password"].asString();
    user.name = value["data"]["name"].asString();
    user.date = value["time"].asInt();
    user.email = value["data"]["email"].asString();
    database::register_return returncode = database::register_(user);
    boost::system::error_code ignore;
    switch (returncode.state) {
    case database::register_state::SUCCESS: {
        // 注册成功
        sprintf(json_tmp,
            "{"
            "  \"code\": 5,"
            "  \"data\": {"
            "    \"state\": 0,"
            "    \"ocId\": \"%s\","
            "    \"id\":%d"
            "  }"
            "}",
            returncode.ocid.c_str(), returncode.id);
        boost::asio::write(socket, boost::asio::buffer(json_tmp), ignore);
        break;
    }
    case database::register_state::EMAIL_DUP: {
        sprintf(json_tmp,
            "{"
            "  \"code\": 5,"
            "  \"data\": {"
            "    \"state\": 2"
            "  }"
            "}");
        boost::asio::write(socket, boost::asio::buffer(json_tmp), ignore);
        break;
    }
    default: {
        sprintf(json_tmp,
            "{"
            "  \"code\": 5,"
            "  \"data\": {"
            "    \"state\": 1"
            "  }"
            "}");
        boost::asio::write(socket, boost::asio::buffer(json_tmp), ignore);
        LOG(ERROR) << "register code " << (int)returncode.state
                   << " is not defined.";
    }
    }
}

server::~server() {
    // 遍历所有socket并释放
    for (const auto& i : clients) {
        delete i.second;
    }
}
}
