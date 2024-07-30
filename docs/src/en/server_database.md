# Database Design of Server

## User Table（user)

| 列名   | 类型            | 用处                                               |
| :----- | :-------------- | :------------------------------------------------- |
| id     | bigint unsigned | 主键，唯一的标识符(使用雪花 id)                    |
| ocid   | char(10)        | 显示给用户的唯一 id 号(使用随机生成的 10 位字符串) |
| passwd | char(64)        | 用户的密码，经过 sha256 加密，不可还原             |
| name   | char(15)        | 昵称                                               |
| email  | CHAR(120)       | 邮箱                                               |
| date   | int             | 时间戳                                             |

## Friend Table(friend)

| 列名      | 类型     | 用处                 |
| :-------- | :------- | :------------------- |
| user_id   | int      | 用户                 |
| friend_id | int      | 用户的好友           |
| name      | char(15) | 用户给好友起的备注名 |

## Chat Table(chat)

| 列名       | 类型     | 用处                           |
| :--------- | :------- | :----------------------------- |
| group_id   | int      | 群聊                           |
| user_id    | int      | 标记用户在该群中               |
| name       | char(15) | 标记用户在群里的昵称           |
| group_name | char(30) | 标记用户对群聊的备注，默认为空 |

## Group Table(group)

| 列名       | 类型     | 用处     |
| :--------- | :------- | :------- |
| group_id   | int      | 群聊 id  |
| group_name | char(30) | 群聊名称 |

## User Chat Message Record Table(user_chat_msg)

| 列名        | 类型 | 用处      |
| :---------- | :--- | :-------- |
| user_id     | int  | 用户的 id |
| chat_msg_id | int  | 信息的 id |

## User Chat Message Detail Table(user_chat_msg_id)

| 列名        | 类型          | 用处          |
| :---------- | :------------ | :------------ |
| char_msg_id | int           | 聊天信息的 id |
| msg_type    | int           | 信息的类型    |
| msg_data    | varchar(8000) | 信息的数据    |
| group_id    | int           | 属于哪个群    |
| sender_id   | int           | 发送者的 id   |
