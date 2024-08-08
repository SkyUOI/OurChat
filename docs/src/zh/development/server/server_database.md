# 服务端的数据库设计

注意：为了方便和文档的准确性，具体数据类型请直接参考[orm entities](https://github.com/SkyUOI/OurChat/tree/main/server/src/entities)

## 用户表(user)

| 列名   | 用处                                               |
| :----- | :------------------------------------------------- |
| id     | 主键，唯一的标识符(使用雪花 id)                    |
| ocid   | 显示给用户的唯一 id 号(使用随机生成的 10 位字符串) |
| passwd | 用户的密码，经过 sha256 加密，不可还原             |
| name   | 昵称                                               |
| email  | 邮箱                                               |
| date   | 时间戳                                             |

## 好友表(friend)

| 列名      | 用处                 |
| :-------- | :------------------- |
| user_id   | 用户                 |
| friend_id | 用户的好友           |
| name      | 用户给好友起的备注名 |

## 聊天表(chat)

| 列名       | 用处                           |
| :--------- | :----------------------------- |
| group_id   | 群聊                           |
| user_id    | 标记用户在该群中               |
| name       | 标记用户在群里的昵称           |
| group_name | 标记用户对群聊的备注，默认为空 |

## 群聊表(groupchat)

| 列名       | 用处     |
| :--------- | :------- |
| group_id   | 群聊 id  |
| group_name | 群聊名称 |

## 用户聊天信息表(user_chat_msg)

| 列名        | 用处      |
| :---------- | :-------- |
| user_id     | 用户的 id |
| chat_msg_id | 信息的 id |

## 聊天信息表(user_chat_id)

| 列名        | 用处          |
| :---------- | :------------ |
| char_msg_id | 聊天信息的 id |
| msg_type    | 信息的类型    |
| msg_data    | 信息的数据    |
| group_id    | 属于哪个群    |
| sender_id   | 发送者的 id   |
