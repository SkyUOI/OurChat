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


# 新建会话请求信息json
**Server <- Client**
```json
{
  "code": 8,
  "time": 时间戳,
  "data": {
    "members": [
      ocid1,
      ocid2,
      ...
    ]
  }
}
```

# 新建会话返回信息json
```json
{
  "code": 9,
  "success": true/false,
  "msg": "失败原因"
}
```