#pragma once

#include <filesystem>
#include <string>

namespace fs = std::filesystem;

namespace ourchat::utils {
/**
 * @brief 读取文件并将字符存入file_data
 * @return 成功返回0，失败返回1
 */
int readfile(std::string& file_data, const std::string& path);
}