# 基本信息json

格式如下

```json
{
  "code": 信息类型,
  "time": 消息发送时的时间戳,
  "data": {}
}
```

| key  | valueType | comment                 |
|:-----|:----------|:------------------------|
| code | int       | 信息类型                    |
| data | json      | 信息相关数据,根据信息所需的数据来定键值对   |
| time | int       | 消息发送时的时间戳               |

# 文本信息json
**Server <-> Client**

格式如下

```json
{
  "code": 0,
  "time": 消息发送时的时间戳,
  "data": {
    "cid": 该信息的id,唯一, //传输给服务器时无此字段
    "sender": {
      "ocid":发送者ocid,
      "group_id":发送此信息的群聊(群聊情况下),
      "private_id":接收此信息的人(私聊情况下)
    },
    "msg": "文本信息"
  }
}
```

| key       | valueType | comment    |
|:----------|:----------|:-----------|
| code      | int       | 信息类型       |
| time      | int       | 消息发送时的时间戳  |
| cid       | int       | chat的ID，唯一 ***(注意：传输给服务器时无此字段)***
| sender    | int       | 发送者的相关数据 |
| ocid      | int       | 发送者的ocid   |
| group_id  | int       | 当会话为群聊时，该值为群号|
| private_id| int       | 当会话为私聊时，该值为对方的ocid|
| data      | json      | 信息相关数据     |
| msg       | str       | 文本信息       |

***code1,2,3分别为还未制作的表情包(包括但不限于gif)，图片发送，文件发送***

# 注册信息json
**Server <- Client**

格式如下

```json
{
  "code": 4,
  "time": 发送请求时的时间戳,
  "data": {
    "email": "注册使用的邮箱",
    "password": "注册密码(已加密)",
    "name": "昵称"
  }
}
```

| key      | valueType | comment   |
|:---------|:----------|:----------|
| code     | int       | 信息类型      |
| time     | int       | 发送请求时的时间戳 |
| data     | json      | 信息相关数据    |
| email     | str       | 注册邮箱      |
| password | str       | 注册密码(已加密)      |
| name     | str       | 昵称        |

# 注册返回信息json
**Server -> Client**

格式如下

```json
{
  "code": 5,
  "time": 时间戳,
  "data": {
    "state": 返回码,
    "ocId": "注册账号的OC号"
  }
}
```

| key   | valueType | comment   |
|:------|:----------|:----------|
| code  | int       | 信息类型      |
| time  | int       | 发送请求时的时间戳 |
| data  | json      | 信息相关数据    |
| state | int       | 服务端返回的状态码 |
| ocId  | int       | 该账号的OC号   |

| returnCode | comment |
|:-----------|:--------|
| 0          | 注册成功    |
| 1          | 服务器错误   |
| 2          | 邮箱重复    |

# 登录信息json
**Server <- Client**

格式如下
```json
{
  "code": 6,
  "time": 发送请求时的时间戳,
  "data": {
    "account": "邮箱/OCID",
    "password": "密码"
  }
}
```

| key      | valueType | comment   |
|:---------|:----------|:----------|
| code     | int       | 信息类型      |
| time     | int       | 发送请求时的时间戳 |
| data     | json      | 信息相关数据    |
| account     | str       | 账号绑定的邮箱或ocId      |
| password | str       | 密码        |

# 登录返回信息json
**Server -> Client**

格式如下

```json
{
  "code": 7,
  "time": 时间戳,
  "data": {
    "state": 登录状态码,
    "ocid":该账号的ocid
  }
}
```

| key   | valueType | comment   |
|:------|:----------|:----------|
| code  | int       | 信息类型      |
| time  | int       | 发送请求时的时间戳 |
| data  | json      | 信息相关数据    |
| state | int       | 服务器返回的状态码 |
| ocid    | int       | 该账号的OCID |

| returnCode | comment  |
|:-----------|:---------|
| 0          | 登录成功     |
| 1          | 账号或密码不正确 |
| 2          | 服务器错误    |

# 建群请求Json
**Server <- Client**

```json
{
  "code": 8,
  "time": 消息发送时的时间戳,
  "data": {
    "group_name": 群名,
    "ocid": 请求建群用户的OCID(群主)
  }
}
```

| key      | valueType | comment |
| -------  | --------- |---------|
|   code   |   int     | 消息类型 |
|  time    |  int      | 时间戳   |
|  data    |  dict     | 相关数据 |
|group_name|  int      | 群聊ID|
|   ocid   |  int      | 请求建群用户的OCID|


# 申请加入群聊/添加好友Json
**Server <-> Client**

格式如下

```json
{
  "code": 9,
  "time": 消息发送时的时间戳,
  "data": {
    "group_id": 该群聊的ID,
    "add_ocid":被申请添加用户的OCID,
    "ocid": 申请人OCID
  }
}
```

| key      | valueType | comment |
| -------  | --------- |---------|
|   code   |   int     | 消息类型 |
|  time    |  int      | 时间戳   |
|  data    |  dict     | 相关数据 |
|group_id  |  int      | 群聊ID(申请群聊时)   |
|   ocid   |  int      | 被申请用户的OCID(申请好友时)   |
|ocid      |   int     |申请人OCID|

# 被拉入群聊Json/通过加入群聊申请/通过好友申请Json
**Server <-> Client**

格式如下

```json
{
  "code": 10,
  "time": 消息发送时的时间戳,
  "data": {
    "group_id": 该群聊的ID,
    "group_name": 群名,
    "add_ocid": 被申请用户的OCID
    "ocid": 发出申请用户的OCID
  }
}
```

| key      | valueType | comment    |
| -------  | --------- |------------|
|   code   |   int     | 消息类型    |
|  time    |  int      | 时间戳      |
|  data    |  dict     | 相关数据    |
|group_id  |  int      |群聊ID(当申请进入群聊时)|
|group_name|   str     |群名(当申请进入群聊时)|
|add_ocid  |    int    |被申请用户的OCID(当申请添加好友时)|
| ocid     |   int     |发出申请用户的OCID|

# 拒绝加入群聊申请/好友申请Json
**Server <-> Client**

格式如下

```json
{
  "code": 11,
  "time": 消息发送时的时间戳,
  "data": {
    "group_id": 该群聊的ID,
    "ocid": 被拒绝用户的OCID
  }
}
```

| key      | valueType | comment |
| -------  | --------- |---------|
|   code   |   int     | 消息类型 |
|  time    |  int      | 时间戳   |
|  data    |  dict     | 相关数据 |
|group_id  |  int      | 群聊ID(当入群申请被拒绝时)   |
|ocid      |   int     | 被拒绝用户的OCID|


# 修改群名/用户昵称Json
**Server <-> Client**

```json
{
  "code": 12,
  "time": 消息发送时的时间戳,
  "data": {
    "group_id": 该群聊的ID,
    "ocid": 该用户OCID,
    "name": 新群名/新昵称
  }
}
```

| key      | valueType |  comment  |
| -------  | --------- |-----------|
|   code   |   int     |  消息类型  |
|  time    |  int      |  时间戳    |
|  data    |  dict     |  相关数据  |
|group_id  |  int      |群聊ID(当群聊改名时)|
|   ocid   |  int      |用户OCID(当用户改名时)|
|group_name|   str     |新群名/新昵称|

