#include <base/filesys.h>
#include <cstdio>
#include <filesystem>
#include <string>
#include <sys/stat.h>

namespace ourchat::utils {
/**
 * @brief 具体的读取文件的细节
 * @param path 文件路径
 * @param file_data 读取出的内容的存放地
 * @param file 文件
 */
inline void read_file_detail(
    const std::string& path, std::string& file_data, FILE* file) {
    struct stat buffer { };
    stat(path.c_str(), &buffer);
    size_t size = buffer.st_size;
    file_data.resize(size + 1);
    file_data[size] = '\0';
    fread((char*)file_data.c_str(), sizeof(char), size, file);
}

int readfile(std::string& file_data, const std::string& path) {
    FILE* file = fopen(path.c_str(), "r");
    if (file == nullptr) {
        return 1;
    }
    read_file_detail(path, file_data, file);
    fclose(file);
    return 0;
}
}
