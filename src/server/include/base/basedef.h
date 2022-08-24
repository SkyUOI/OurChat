#pragma once
#include <mysql.h>
#include <string>

namespace ourchat::database {
extern MYSQL mysql;

// 重复数据
constexpr unsigned int DUP_DATA = 1062;

/**
 * @brief 初始化数据库
 * @param dbconfig_path 标识数据库配置文件的路径
 */
void init(const std::string& dbconfig_path);

void quit();
}