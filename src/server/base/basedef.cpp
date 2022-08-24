#include <base/filesys.h>
#include <cstdlib>
#include <easylogging++.h>
#include <json/json.h>
#include <mysql.h>

namespace ourchat::database {
MYSQL mysql;

std::string host;
std::string user;
std::string passwd;
std::string db;
unsigned int port;

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
            "passwd CHAR(30),"
            "name CHAR(15),"
            "email CHAR(120),"
            "date INT,"
            "PRIMARY KEY(id),"
            "UNIQUE KEY(ocid)"
            ")Engine=InnoDB DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the user table";
    }
    // friend
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS friend("
            "user_id INT,"
            "friend_id INT,"
            "name CHAR(15)"
            ")Engine=InnoDB DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the friend table";
    }
    // chat
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS chat("
            "group_id INT,"
            "user_id INT,"
            "name CHAR(15),"
            "group_name CHAR(30)"
            ")Engine=InnoDB DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the chat table";
    }
    // chatgroup
    if (mysql_query(&mysql,
            "CREATE TABLE IF NOT EXISTS chatgroup("
            "group_id INT,"
            "group_name CHAR(30)"
            ")Engine=InnoDB DEFAULT CHARSET=utf8mb4;")) {
        LOG(FATAL) << "Error in creating the chatgroup table";
    }
}

void quit() {
    mysql_close(&mysql);
}
}
