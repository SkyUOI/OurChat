/**
 * @brief 登录数据库读写
 */

#include <base/basedef.h>
#include <base/data.h>
#include <base/login.h>
#include <easylogging++.h>
#include <mysql.h>

namespace ourchat::database {
login_state login(const std::string& account, const std::string& password) {
    // 第一步查询账号
    return login_state::SUCCESS;
}

// 保存ocid的数组
char register_sql_tmp[300];
// 储存ocid的各种可能出现的组合
char ocid_char_list[] = { 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9' };

constexpr unsigned int ocid_default_size = 14;

std::string make_random_ocid() {
    std::string ocid;
    ocid.reserve(ocid_default_size);
    for (unsigned int i = 0; i < ocid_default_size; ++i) {
        ocid += ocid_char_list[utils::random(
            0, utils::sizeof_static_array(ocid_char_list))];
    }
    return ocid;
}

register_return register_(const user_for_register& user) {
    do {
        size_t len = sprintf(register_sql_tmp,
            "INSERT INTO user(ocid CHAR(20),"
            "passwd CHAR(30),"
            "name CHAR(15),"
            "email CHAR(120),"
            "date INT) VALUES (%s %s %s %d)",
            make_random_ocid().c_str(), user.passwd.c_str(), user.email.c_str(),
            user.date);
        if (mysql_real_query(&mysql, register_sql_tmp, len)) {
            unsigned int errcode = mysql_errno(&mysql);
            switch (errcode) {
            case DUP_DATA: {
                // 数据重复，重新生成
                continue;
            }
            default: {
                LOG(ERROR) << "Can't insert data " << user.email << ' '
                           << user.name << ' ' << user.passwd << ' '
                           << mysql_error(&mysql);
                return { register_state::DATABASE_ERROR };
            }
            }
        }
        break;
    } while (true);
    MYSQL_RES* res = mysql_store_result(&mysql);
    register_return returndata = { register_state::SUCCESS };

    mysql_free_result(res);
    return returndata;
}
}
