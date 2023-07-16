#include <base/filesys.h>
#include <easylogging++.h>
#include <json/json.h>
#include <mysql/mysql.h>

namespace ourchat::database {
MYSQL mysql;

std::string host;
std::string user;
std::string passwd;
std::string db;
unsigned int port;

char sql[10201];

void init(const std::string& dbconfig_path) {
    // 初始化数据库
    mysql_init(&mysql);
    std::string filedata;
    if (utils::readfile(filedata, dbconfig_path)) {
        LOG(FATAL) << "Can't find " << dbconfig_path;
    }
    Json::Value jsondata;
    Json::Reader reader;
    reader.parse(filedata, jsondata);
    host = jsondata["host"].asString();
    user = jsondata["user"].asString();
    passwd = jsondata["passwd"].asString();
    db = jsondata["db"].asString();
    // 连接数据库
    if (!mysql_real_connect(&mysql, host.c_str(), user.c_str(), passwd.c_str(),
            db.c_str(), port, nullptr, 0)) {
        LOG(FATAL) << "Can't connect to the mysql database " << host;
    }
    // 创建几张表
    // user
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS user("
            "id INT AUTO_INCREMENT,"
            "ocid CHAR(20),"
            "passwd CHAR(64),"
            "name CHAR(15),"
            "email CHAR(120),"
            "date INT,"
            "PRIMARY KEY(id),"
            "UNIQUE KEY(ocid),"
            "UNIQUE KEY(email)"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the user table" << mysql_errno(&mysql)
                   << mysql_error(&mysql);
    }
    // friend
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS friend("
            "user_id INT,"
            "friend_id INT,"
            "name CHAR(15),"
            "PRIMARY KEY(user_id)"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the friend table"
                   << mysql_errno(&mysql) << mysql_error(&mysql);
    }
    // chat
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS chat("
            "group_id INT AUTO_INCREMENT,"
            "user_id INT,"
            "name CHAR(15),"
            "group_name CHAR(30),"
            "PRIMARY KEY(group_id)"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the chat table" << mysql_errno(&mysql)
                   << mysql_error(&mysql);
    }
    // chatgroup
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS chatgroup("
            "group_id INT,"
            "group_name CHAR(30),"
            "PRIMARY KEY(group_id)"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the chatgroup table"
                   << mysql_errno(&mysql) << mysql_error(&mysql);
    }
    // user_chat_msg
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS user_chat_msg("
            "user_id INT NOT NULL,"
            "chat_msg_id INT NOT NULL"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the user_chat_msg table"
                   << mysql_errno(&mysql) << mysql_error(&mysql);
    }
    // user_chat_id
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS user_chat_id("
            "chat_msg_id INT AUTO_INCREMENT,"
            "msg_type INT,"
            "msg_data VARCHAR(8000),"
            "sender_id INT,"
            "PRIMARY KEY(chat_msg_id)"
            ")DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the user_chat_id table"
                   << mysql_errno(&mysql) << mysql_error(&mysql);
    }
}

void quit() {
    mysql_close(&mysql);
}
}
