#pragma once
#include <mysql.h>
#include <server/server_def.h>
#include <string>
#include <vector>

namespace ourchat::database {
/**
 * @brief 通过组的id获取里面的所有人数
 * @param group_id
 * @return
 */
MYSQL_RES* get_members_by_group(group_id_t group_id);

enum class msg_type_t { TEXT, IMAGE };

/**
 * @brief 向对应用户保存信息
 * @param user 用户id
 * @param msg_id 数据id
 */
void save_chat_msg(user_id_t user, msg_id_t msg_id);

/**
 * @brief 将一条信息保存到数据库中
 * @param json 数据
 * @return 该数据对应的数据库id,返回-1代表出现错误
 */
int saved_msg(const std::string& json, group_id_t group_id, user_id_t sender_id,
    msg_type_t msg_type);
}