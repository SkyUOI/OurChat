#pragma once
#include <cstdio>
#include <string>

/**
 * @param str 报错的辅助输出信息
 */
#define NOREACH(str, ...)                                                      \
    do {                                                                       \
        printf(str "\n", __VA_ARGS__);                                         \
        ourchat::noreach_internal(__LINE__, __FUNCTION__, __FILE__);           \
    } while (0)

namespace ourchat {
extern const int port;

enum class client_code {
    TEXT = 0,
    EMOJI = 1,
    PICTURE = 2,
    FILE = 3,
    REGISTER = 4,
    LOGIN = 6
};

/**
 * @brief
 * 在不能被执行到的地方放上这条语句，出现问题会强行停止程序
 */
void noreach_internal(size_t line, const char* funcname, const char* filename);

typedef unsigned int group_id_t;

typedef std::string ocid_t;
}
