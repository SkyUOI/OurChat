/**
 * @brief 登录数据库读写
 */

#include <base/basedef.h>
#include <base/data.h>
#include <base/login.h>
#include <easylogging++.h>
#include <mysql.h>

namespace ourchat::database {
login_return login(const std::string& account, const std::string& password) {
    // 第一步查询账号
    return { login_state::SUCCESS };
}

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
    sprintf(sql, "select COUNT(email) from user where email=\"%s\"",
        user.email.c_str());
    if (mysql_query(&mysql, sql)) {
        LOG(ERROR) << "Can't select data from db." << mysql_errno(&mysql) << ' '
                   << mysql_error(&mysql);
        return { register_state::DATABASE_ERROR };
    }
    MYSQL_RES* res = mysql_store_result(&mysql);
    MYSQL_ROW count_data = mysql_fetch_row(res);
    if (count_data[0][0] != '0') {
        mysql_free_result(res);
        return { register_state::EMAIL_DUP };
    }
    mysql_free_result(res);
    std::string ocid;
    do {
        ocid = make_random_ocid().c_str();
        size_t len = sprintf(sql,
            "INSERT INTO user(ocid,"
            "passwd,"
            "name,"
            "email,"
            "date) VALUES (\"%s\", \"%s\",\"%s\", \"%s\", %d)",
            ocid.c_str(), user.passwd.c_str(), user.name.c_str(),
            user.email.c_str(), user.date);
        if (mysql_real_query(&mysql, sql, len)) {
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
    // 获取刚刚插入的数据
    mysql_query(&mysql, "select last_insert_id();");
    res = mysql_store_result(&mysql);
    MYSQL_ROW last_insert_id = mysql_fetch_row(res);
    register_return returndata
        = { register_state::SUCCESS, ocid, atoi(last_insert_id[0]) };
    mysql_free_result(res);
    return returndata;
}
}
