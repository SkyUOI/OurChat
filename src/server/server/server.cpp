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
            break;
        }
        default: {
            LOG(ERROR) << "client code " << int(receive_code)
                       << " is not defined.";
        }
        }
    }
}

static std::string make_login_json(const std::string& state_code) {
    std::string s = R"({"code":7, "data":{"state":)";
    s += state_code;
    s += "}}";
    return s;
}

void server::trylogin(tcp::socket& socket, Json::Value value) {
    boost::system::error_code error;
    const std::string& ocid = value["data"]["ocId"].asString();
    const std::string& passwd = value["data"]["password"].asString();
    database::login_state return_code = database::login(ocid, passwd);
    switch (return_code) {
    case database::login_state::SUCCESS: {
        // 正常登录
        boost::asio::write(
            socket, boost::asio::buffer(make_login_json("0")), error);
        // 保存套接字
        clients[ocid] = &socket;
        break;
    }
    case database::login_state::PASSWORDINCORRECT:
    case database::login_state::ACCOUNTNOTFOUND: {
        // 账号未定义或密码错误
        boost::asio::write(
            socket, boost::asio::buffer(make_login_json("1")), error);
        break;
    }
    default: {
        LOG(ERROR) << "login code " << int(return_code) << " is not defined.";
        boost::asio::write(
            socket, boost::asio::buffer(make_login_json("2")), error);
    }
    }
}

void server::send_text(const std::string& json, group_id_t group) {
    // 首先根据group_id获取群聊的所有人数，然后按照OCID发送给具体的socket，未存在socket链接则储存信息
    boost::system::error_code error;
    for (const auto& i : database::get_members_by_group(group)) {
        if (clients.find(i) != clients.end()) {
            // 存在socket链接
            boost::asio::write(*(clients[i]), boost::asio::buffer(json), error);
            // 判断是否成功发送
            if (error == boost::asio::error::eof) {
                // 连接已结束保存数据到数据库,等待下一次发送
                database::save_chat_msg(i, json);
                delete clients[i];
                clients.erase(i);
            }
        } else {
            // 不存在socket链接，保存数据到数据库,等待下一次发送
            database::save_chat_msg(i, json);
        }
    }
}

/**
 * @brief 制造登录返回的信息
 * @param db_return 数据库返回的数据
 * @param state_code 状态码
 * @return json
 */
std::string make_register_json(
    const database::register_return& db_return, const std::string& state_code) {
    std::string s = "";
}

void server::tryregister(tcp::socket& socket, Json::Value value) {
    // 通过json解析出数据
    database::user_for_register user;
    user.passwd = value["data"]["passwd"].asString();
    user.name = value["data"]["passwd"].asString();
    user.date = value["data"]["date"].asInt();
    user.email = value["data"]["email"].asString();
    database::register_return returncode = database::register_(user);
    boost::system::error_code ignore;
    switch (returncode.state) {
    case database::register_state::SUCCESS: {
        boost::asio::write(
            socket, boost::asio::buffer(make_register_json()), ignore);
        break;
    }
    default: {
        boost::asio::write(
            socket, boost::asio::buffer(make_register_json()), ignore);
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
