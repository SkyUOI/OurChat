/**
 * 登录的数据库接口
 */
#pragma once

#include <string>

namespace ourchat::database {
enum class login_state {
    SUCCESS, // 正常登录
    ACCOUNTNOTFOUND, // 账号找不到
    PASSWORDINCORRECT, // 密码不正确
};

/**
 * @brief 进行登录操作
 */
login_state login(const std::string& account, const std::string& password);
}