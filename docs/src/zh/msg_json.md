# OurChat 信息传递格式

## 目录

- [文本信息](#文本信息)
- [注册信息](#注册信息)
- [注册返回信息](#注册返回信息)
- [登录信息](#登录信息)
- [登录返回信息](#登录返回信息)
- [新建会话请求信息](#新建会话请求信息)
- [新建会话返回信息](#新建会话返回信息)
- [获取账号信息](#获取账号信息)
- [获取账号信息返回信息](#获取账号信息返回信息)
- [获取服务器状态](#获取服务器状态)
- [发起验证](#发起验证)
- [生成验证码](#生成验证码)
- [验证状态](#验证状态)

## 文本信息

**_Server <-> Client_**

格式如下

```json
{
  "code": 0,
  "time": 消息发送时的时间戳,
  "msg_id": 该信息的id,唯一, //传输给服务器时无此字段
  "sender": {
    "ocid":发送者ocid,
    "session_id":发送此信息的会话id,
  },
  "msg": "文本信息"
}
```

| key        | valueType | comment                                                        |
| :--------- | :-------- | :------------------------------------------------------------- |
| code       | Number    | 信息类型                                                       |
| time       | Number    | 发消息的时间戳                                                 |
| msg_id     | Number    | message 的 ID，唯一 **_(注意：传输给服务器时无此字段)_**          |
| sender     | Object    | 发送者的相关数据                                               |
| ocid       | String    | 发送者的 ocid                                                  |
| session_id | Number    | 发送者的会话 id                                                |
| msg        | String    | 文本信息                                                       |

**_code1,2,3 分别为还未制作的表情包(包括但不限于 gif)，图片发送，文件发送_**

## 注册信息

**_Server <- Client_**

格式如下

```json
{
  "code": 4,
  "email": "注册使用的邮箱",
  "password": "注册密码(已加密)",
  "name": "昵称"
}
```

| key      | valueType | comment          |
| :------- | :-------- | :--------------- |
| code     | Number    | 信息类型         |
| email    | String    | 注册邮箱         |
| password | String    | 注册密码(已加密) |
| name     | String    | 昵称             |

## 注册返回信息

**_Server -> Client_**

格式如下

```json
{
  "code": 5,
  "status": 返回码,
  "ocid": "注册账号的OC号"
}
```

| key    | valueType | comment            |
| :----- | :-------- | :----------------- |
| code   | Number    | 信息类型           |
| status | Number    | 服务端返回的状态码 |
| ocid   | Number    | 该账号的 OC 号     |

| returnCode | comment    |
| :--------- | :--------- |
| 0          | 注册成功   |
| 1          | 服务器错误 |
| 2          | 邮箱重复   |

## 登录信息

**_Server <- Client_**

格式如下

```json
{
  "code": 6,
  "login_type": 登陆方式,
  "account": "邮箱/OCID",
  "password": "密码"
}
```

| key        | valueType | comment                      |
| :--------- | :-------- | :--------------------------- |
| code       | Number    | 信息类型                     |
| login_type | Number    | 0 为邮箱登录，1 为 ocid 登录 |
| account    | String    | 账号绑定的邮箱或 ocid        |
| password   | String    | 密码                         |

## 登录返回信息

**_Server -> Client_**

格式如下

```json
{
  "code": 7,
  "status": 登录状态码,
  "ocid":该账号的ocid
}
```

| key    | valueType | comment            |
| :----- | :-------- | :----------------- |
| code   | Number    | 信息类型           |
| status | Number    | 服务器返回的状态码 |
| ocid   | Number    | 该账号的 OCID      |

| status     | comment          |
| :--------- | :--------------- |
| 0          | 登录成功         |
| 1          | 账号或密码不正确 |
| 2          | 服务器错误       |

## 新建会话请求信息

**_Server <- Client_**

```json
{
  "code": 8,
  "members": [
    "ocid1",
    "ocid2",
    ...
  ]
}
```

| key     | valueType | comment  |
| :------ | :-------- | :------- |
| code    | Number    | 信息类型 |
| members | Array     | 会话成员 |

## 新建会话返回信息

**_Server -> Client_**

```json
{
  "code": 9,
  "status": 会话状态码,
  "session_id": 会话id // 仅当创建成功时有此字段
}
```

| key      | valueType | comment          |
| :------  | :-------- | :--------------- |
| code     | Number    | 信息类型          |
| status   | Number    | 会话状态码        |
|session_id| Number    | 会话id            |

| status     | comment          |
| :--------- | :--------------- |
| 0          | 创建成功          |
| 1          | 服务器错误        |
| 2          | 到达创建会话上限   |

## 获取账号信息

**_Server <- Client_**

```json
{
  "code": 10,
  "ocid": 该账号的OCID,
  "request_values":[
    "ocid",
    "nickname",
    ...
  ]
}
```

| key            | valueType | comment              |
| :------------- | :-------- | :------------------- |
| code           | Number    | 信息类型             |
| ocid           | Number    | 该账号的 ocid        |
| request_values | Array     | 需要服务端返回的信息 |

| request_value | comment               |
| :------------ | :-------------------- |
| ocid          | 该账号的 ocid         |
| nickname      | 昵称                  |
| status        | 该账号的状态          |
| avater        | 该账号头像的 url 链接 |
| time          | 该账号注册的时间戳    |

## 获取账号信息返回信息

**_Server -> Client_**

```json
{
  "code": 11,
  "data":{
    "ocid": 该账号的OCID,
    "nickname": 昵称,
    ...
  }
}
```

| key  | valueType | comment                                           |
| :--- | :-------- | :------------------------------------------------ |
| code | Number    | 信息类型                                          |
| data | Object    | 账号信息,详情[见上`request_value`](#获取账号信息) |

## 获取服务器状态

**_Server <-> Client_**

```json
{
  "code": 12,
  "status": 服务器状态码, // 传输给服务器时无此字段
}
```

| key  | valueType | comment           |
| :--- | :-------- | :---------------- |
| code | Number    | 信息类型           |
|status| Number    | 服务器状态码       |

| status | comment  |
| :----- | :------- |
| 0      | 正常运行  |
| 1      | 维护中   |

## 发起验证

**_Server -> Client_**

```json
{
  "code": 13
}
```

| key  | valueType | comment           |
| :--- | :-------- | :---------------- |
| code | Number    | 信息类型           |

## 生成验证码

**_Server <- Client_**

```json
{
  "code": 14
}
```

## 验证状态

**_Server -> Client_**

```json
{
  "code": 15,
  "status": 验证状态码
}
```

| key  | valueType | comment           |
| :--- | :-------- | :---------------- |
| code | Number    | 信息类型           |
|status| Number    | 验证状态码         |

| status | comment  |
| :----- | :------- |
| 0      | 验证通过 |
| 1      | 验证失败 |
| 2      | 验证超时 |
