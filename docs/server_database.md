# 服务端的数据库设计

## 用户表
### user

| 列名     | 类型        | 用处                    |
|:-------|:----------|:----------------------|
| id     | int       | 主键，唯一的标识符             |
| ocid   | char(20)  | 显示给用户的唯一id号           |
| passwd | char(64)  | 用户的密码，经过sha256加密，不可还原 |
| name   | char(15)  | 昵称                    |
| email  | CHAR(120) | 邮箱                    |
| date   | int       | 时间截                   |

## 好友表
### friend
| 列名        | 类型       | 用处         |
|:----------|:---------|:-----------|
| user_id   | int      | 用户         |
| friend_id | int      | 用户的好友      |
| name      | char(15) | 用户给好友起的备注名 |

## 聊天表
### chat
| 列名         | 类型       | 用处              |
|:-----------|:---------|:----------------|
| group_id   | int      | 群聊              |
| user_id    | int      | 标记用户在该群中        |
| name       | char(15) | 标记用户在群里的昵称      |
| group_name | char(30) | 标记用户对群聊的备注，默认为空 |

## 群聊表
### groupchat
| 列名         | 类型       | 用处   |
|:-----------|:---------|:-----|
| group_id   | int      | 群聊id |
| group_name | char(30) | 群聊名称 |

## 用户聊天信息表
### user_chat_msg
| 列名          | 类型  | 用处    |
|:------------|:----|:------|
| user_id     | int | 用户的id |
| chat_msg_id | int | 信息的id |

## 聊天信息表
### user_chat_id
| 列名          | 类型            | 用处      |
|:------------|:--------------|:--------|
| char_msg_id | int           | 聊天信息的id |
| msg_type    | int           | 信息的类型   |
| msg_data    | varchar(8000) | 信息的数据   |
| group_id    | int           | 属于哪个群   |
| sender_id   | int           | 发送者的id  |
