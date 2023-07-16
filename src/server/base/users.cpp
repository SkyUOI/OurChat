#include <base/basedef.h>
#include <base/users.h>
#include <easylogging++.h>
#include <mysql/mysql.h>
#include <string>

namespace ourchat::database {
MYSQL_RES* get_members_by_group(group_id_t group_id) {
    sprintf(sql, "select user_id from chat where group_id=%d", group_id);
    if (mysql_query(&mysql, sql)) {
        LOG(ERROR) << "get members by group " << mysql_errno(&mysql)
                   << mysql_error(&mysql);
        return nullptr;
    }
    return mysql_store_result(&mysql);
}

void save_chat_msg(user_id_t user, msg_id_t msg_id) {
    sprintf(sql,
        "insert INTO user_char_msg (user_id, chat_msg_id) VALUES (%s %s);",
        user, msg_id);
    if (!mysql_query(&mysql, sql)) {
        LOG(ERROR) << "Can't save chat msg for user " << mysql_errno(&mysql)
                   << mysql_error(&mysql);
    }
}

int saved_msg(const std::string& json, group_id_t group_id, user_id_t sender_id,
    msg_type_t msg_type) {
    sprintf(sql,
        "INSERT INTO user_char_id (msg_type, msg_data, sender_id) VALUES (%d, "
        "\"%s\", %d);",
        msg_type, json.c_str(), sender_id);
    if (mysql_query(&mysql, sql)) {
        LOG(ERROR) << "Can't save chat msg " << mysql_errno(&mysql)
                   << mysql_error(&mysql);
        return -1;
    }
    // 插入成功,返回id
    MYSQL_RES* res = mysql_store_result(&mysql);
    int msg_id = atoi(mysql_fetch_row(res)[0]);
    mysql_free_result(res);
    return msg_id;
}
}
