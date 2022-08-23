#pragma once
#include <mysql.h>
#include <string>

namespace ourchat::database {
extern MYSQL* mysql;

/**
 * @brief 初始化数据库
 * @param dbconfig_path 标识数据库配置文件的路径
 */
void init(const std::string& dbconfig_path);

void quit();
}