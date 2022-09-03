#pragma once
#include <server/server_def.h>
#include <string>
#include <vector>

namespace ourchat {
namespace database {
    std::vector<ocid_t> get_members_by_group(group_id_t group_id);

    void save_chat_msg(ocid_t user, const std::string& json);
}
}