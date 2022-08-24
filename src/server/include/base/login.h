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

enum class register_state {
    SUCCESS, // 成功
    DATABASE_ERROR // 数据库错误
};

/**
 * @brief 用来注册的user类
 */
struct user_for_register {
    std::string name;
    std::string passwd;
    std::string email;
    int date;
};

/**
 * @brief 用来注册的user类
 */
struct register_return {
    register_state state;
    std::string ocid;
    int id;
};

/**
 * @brief 进行注册操作
 */
register_return register_(const user_for_register& user);
}
