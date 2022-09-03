#pragma once
#include <boost/asio.hpp>
#include <mysql.h>
#include <string>

namespace ourchat::database {
extern MYSQL mysql;

// 重复数据
constexpr unsigned int DUP_DATA = 1062;

// 一条sql语句
extern char sql[10201];

/**
 * @brief 初始化数据库
 * @param dbconfig_path 标识数据库配置文件的路径
 */
void init(const std::string& dbconfig_path);

void quit();
}