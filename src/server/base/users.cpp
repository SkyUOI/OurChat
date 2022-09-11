#include <base/basedef.h>
#include <boost/asio.hpp>
#include <easylogging++.h>
#include <mysql.h>
#include <server/server_def.h>
#include <string>
#include <vector>

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

void save_chat_msg(int user, int chat_id) {
    sprintf(sql, "insert INTO ");
}

int saved_msg(const std::string& json) {
    sprintf(sql, "INSERT INTO ");
}
}
