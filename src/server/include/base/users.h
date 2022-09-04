#pragma once
#include <mysql.h>
#include <server/server_def.h>
#include <string>
#include <vector>

namespace ourchat::database {
MYSQL_RES* get_members_by_group(group_id_t group_id);

void save_chat_msg(int user, const std::string& json);
}