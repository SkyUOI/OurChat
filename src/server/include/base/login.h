/**
 * 登录的数据库接口
 */
#pragma once

#include <base/basedef.h>
#include <easylogging++.h>
#include <mysql.h>
#include <string>

namespace ourchat::database {
enum class login_state {
    SUCCESS, // 正常登录
    ACCOUNTNOTFOUND, // 账号找不到
    PASSWORDINCORRECT, // 密码不正确
    DATABASEERROR // 数据库异常
};

/**
 * @brief 登录的返回值
 */
struct login_return {
    login_state state;
    int id;
};

/**
 * @brief 进行登录操作
 * @tparam logintype 登录类型，true为邮箱，false为ocid
 */
template <bool logintype>
login_return login(const std::string& account, const std::string& password) {
    if constexpr (logintype) {
        sprintf(
            sql, "select passwd from user where email = %s", account.c_str());
    } else {
        sprintf(
            sql, "select passwd from user where ocid = %s", account.c_str());
    }

    if (mysql_query(&mysql, sql)) {
        LOG(ERROR) << "login with ocid error " << mysql_errno(&mysql)
                   << mysql_error(&mysql);
        return { login_state::DATABASEERROR };
    }
    MYSQL_RES* res = mysql_store_result(&mysql);
    MYSQL_ROW row = mysql_fetch_row(res);
    login_return ret;
    if (row == nullptr) {
        // 没有找到该账号
        ret.state = login_state::ACCOUNTNOTFOUND;
    } else if (!strcmp(row[0], password.c_str())) {
        // 如果相等,登陆成功
        ret.state = login_state::SUCCESS;
    } else {
        // 密码错误，登录失败
        ret.state = login_state::PASSWORDINCORRECT;
    }
    mysql_free_result(res);
    return ret;
}

enum class register_state {
    SUCCESS, // 成功
    DATABASE_ERROR, // 数据库错误
    EMAIL_DUP // 邮箱重复
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
