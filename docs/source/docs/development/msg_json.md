# OurChat 信息传递格式

## 用户信息

**_Server <-> Client_**

```json
// E.g.
{
  "code": 0,
  "time": 114514, // 发送消息的时间戳
  "msg_id": "1643212388", //传输给服务器时无此字段
  "sender": {
    "ocid":"0000000000",
    "session_id": "1145141919" //发送此消息的会话id
  },
  "msg": [
    {
      "type":0, // 用户消息类型
      // ...相关数据
    },
    // ...
  ]
}
```

| key        | ValueType | comment                                                 |
|:-----------|:----------|:--------------------------------------------------------|
| code       | Number    | 信息类型                                                |
| time       | Number    | 发消息的时间戳                                          |
| msg_id     | Number    | message 的 ID，唯一 **_(注意：传输给服务器时无此字段)_**  |
| sender     | Object    | 发送者的相关数据                                        |
| ocid       | String    | 发送者的 ocid                                           |
| session_id | Number    | 发送者的会话 id                                         |
| msg        | Array     | 消息列表                                                |
| type       | Number    | 用户消息类型，详细见[用户消息传递格式](user_msg_json.md) |

## 获取会话信息

**_Server <- Client_**

```json
// E.g.
{
  "code": 1,
  "session_id": "1145141919", // 该会话的ID,
  "request_values":[
    "name",
    // ...
  ]
}
```

| key            | ValueType | comment              |
|:---------------|:----------|:---------------------|
| code           | Number    | 信息类型              |
| session_id     | String    | 该会话的ID            |
| request_values | Array     | 需要服务端返回的信息   |

| request_value | comment                                   |
|:--------------|:------------------------------------------|
| session_id    | 该会话的ID                                 |
| name          | 会话名称                                   |
| avatar        | 该会话头像的 url 链接                       |
| avatar_hash   | 该会话头像的哈希值                          |
| time          | 该会话创建的时间戳                          |
| update_time   | 该会话数据最后更新的时间戳                   |
| members       | 该会话的成员列表                            |
| owner         | 该会话拥有者的 ocid                         |

## 获取会话信息返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 2,
  "data":{
    "session_id": "1145141919", // 该会话的ID
    "name": "Session1", // 会话名称
    // ...
  }
}
```

| key  | ValueType | comment                                           |
|:-----|:----------|:--------------------------------------------------|
| code | Number    | 信息类型                                          |
| data | Object    | 账号信息,详情[见上**获取会话信息**`request_value`](#获取会话信息) |

## 注册信息

**_Server <- Client_**

```json
// E.g.
{
  "code": 4,
  "email": "123456@ourchat.com", // 注册使用的邮箱
  "password": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", // 注册密码(已加密)
  "name": "Outchat" // 昵称
}
```

| key      | ValueType | comment          |
|:---------|:----------|:-----------------|
| code     | Number    | 信息类型         |
| email    | String    | 注册邮箱         |
| password | String    | 注册密码(已加密) |
| name     | String    | 昵称             |

## 注册返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 5,
  "status": 0, // 返回码
  "ocid": "0000000000" // 注册账号的OCID
}
```

| key    | ValueType | comment            |
|:-------|:----------|:-------------------|
| code   | Number    | 信息类型           |
| status | Number    | 服务端返回的状态码 |
| ocid   | String    | 该账号的 OC 号     |

| returnCode | comment    |
|:-----------|:-----------|
| 0          | 注册成功   |
| 1          | 服务器错误 |
| 2          | 邮箱重复   |

## 登录信息

**_Server <- Client_**

```json
// E.g.
{
  "code": 6,
  "login_type": 1, // 登录方式,此处1表示使用ocid登录
  "account": "0000000000", // 邮箱/OCID
  "password": "密码"
}
```

| key        | ValueType | comment                     |
|:-----------|:----------|:----------------------------|
| code       | Number    | 信息类型                    |
| login_type | Number    | 0 为邮箱登录，1 为 ocid 登录 |
| account    | String    | 账号绑定的邮箱或 ocid       |
| password   | String    | 密码                        |

## 登录返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 7,
  "status": 0, // 登录状态码
  "ocid":"0000000000" // 该账号的ocid
}
```

| key    | ValueType | comment            |
|:-------|:----------|:-------------------|
| code   | Number    | 信息类型           |
| status | Number    | 服务器返回的状态码 |
| ocid   | String    | 该账号的 OCID      |

| status | comment          |
|:-------|:-----------------|
| 0      | 登录成功         |
| 1      | 账号或密码不正确 |
| 2      | 服务器错误       |

## 新建会话请求信息

**_Server <- Client_**

```json
// E.g.
{
  "code": 8,
  "members": [
    "0000000000",
    "0000000001",
    // ...
  ]
}
```

| key     | ValueType | comment  |
|:--------|:----------|:---------|
| code    | Number    | 信息类型 |
| members | Array     | 会话成员 |

## 新建会话返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 9,
  "status": 0, // 会话状态码
  "session_id": "1145141919" // 仅当创建成功时有此字段
}
```

| key        | ValueType | comment    |
|:-----------|:----------|:-----------|
| code       | Number    | 信息类型   |
| status     | Number    | 会话状态码 |
| session_id | Number    | 会话 id    |

| status | comment          |
|:-------|:-----------------|
| 0      | 创建成功         |
| 1      | 服务器错误       |
| 2      | 到达创建会话上限 |

## 获取账号信息

**_Server <- Client_**

```json
// E.g.
{
  "code": 10,
  "ocid": "0000000000", //该账号的OCID
  "request_values":[
    "ocid",
    "nickname",
    // ...
  ]
}
```

| key            | ValueType | comment              |
|:---------------|:----------|:---------------------|
| code           | Number    | 信息类型             |
| ocid           | String    | 该账号的 ocid        |
| request_values | Array     | 需要服务端返回的信息 |

| request_value | comment                                   |
|:--------------|:------------------------------------------|
| ocid          | 该账号的 ocid                             |
| nickname      | 昵称                                      |
| status        | 该账号的状态                              |
| avatar        | 该账号头像的 url 链接                     |
|avatar_hash    | 该账号头像的 hash                         |
| time          | 该账号注册的时间戳                        |
| update_time   | 该账号数据最后更新的时间戳                 |
| sessions      | 该账号加入/创建的会话列表(仅本账号可获取)   |
| friends       | 该账号的好友列表 (仅本账号可获取)          |

## 获取账号信息返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 11,
  "data":{
    "ocid": "0000000000", // 该账号的 ocid
    "nickname": "OurChat", // 该账号的昵称
    // ...
  }
}
```

| key  | ValueType | comment                                           |
|:-----|:----------|:--------------------------------------------------|
| code | Number    | 信息类型                                          |
| data | Object    | 账号信息,详情[见上**获取账号信息**`request_value`](#获取账号信息) |

## 获取服务器状态

**_Server <-> Client_**

```json
// E.g.
{
  "code": 12,
  "status": 0 // 服务器状态码,传输给服务器时无此字段
}
```

| key    | ValueType | comment      |
|:-------|:----------|:-------------|
| code   | Number    | 信息类型     |
| status | Number    | 服务器状态码 |

| status | comment  |
|:-------|:---------|
| 0      | 正常运行 |
| 1      | 维护中   |

## 发起验证

**_Server -> Client_**

```json
// E.g.
{
  "code": 13
}
```

| key  | ValueType | comment  |
|:-----|:----------|:---------|
| code | Number    | 信息类型 |

## 生成验证码

**_Server <- Client_**

```json
// E.g.
{
  "code": 14
}
```

## 验证状态

**_Server -> Client_**

```json
// E.g.
{
  "code": 15,
  "status": 0 // 验证状态码
}
```

| key    | ValueType | comment    |
|:-------|:----------|:-----------|
| code   | Number    | 信息类型   |
| status | Number    | 验证状态码 |

| status | comment  |
|:-------|:---------|
| 0      | 验证通过 |
| 1      | 验证失败 |
| 2      | 验证超时 |

## 注销

**_Server<-Client_**

```json
// E.g.
{
  "code": 16
}
```

**_警告：该注销是删除帐号的意思，请勿误用接口_**

## 注销返回信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 17,
  "status": 0 // 注销状态码
}
```

| key    | ValueType | comment    |
|:-------|:----------|:-----------|
| code   | Number    | 信息类型   |
| status | Number    | 注销状态码 |

| status | comment  |
|:-------|:---------|
| 0      | 注销成功 |
| 1      | 注销失败 |

## 错误信息

**_Server -> Client_**

```json
// E.g.
{
  "code": 18,
  "details": "错误信息"
}
```

| Key     | ValueType | Comment  |
|:--------|:----------|:---------|
| code    | Number    | 信息类型 |
| details | String    | 异常信息 |
