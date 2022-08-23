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

/**
 * @brief 遍历目录
 */
class listfiles {
public:
    /**
     * @param path 遍历的目录
     */
    listfiles(const std::string& path);

    /**
     * @return 获取下一项，为空字符串说明到头了
     */
    fs::path nextitem();

private:
    fs::recursive_directory_iterator iter, now;
};
}