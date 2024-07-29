# OurChat 信息传递格式

## 目录

- [基本信息](#基本信息-json)
- [文本信息](#文本信息-json)
- [注册信息](#注册信息-json)
- [注册返回信息](#注册返回信息-json)
- [登录信息](#登录信息-json)
- [登录返回信息](#登录返回信息-json)
- [新建会话请求信息](#新建会话请求信息-json)
- [新建会话返回信息](#新建会话返回信息-json)
- [获取账号信息返回](#获取账号信息返回-json)

## 基本信息 json

格式如下

```json
{
  "code": 信息类型,
  "time": 消息发送时的时间戳,
  ...
}
```

| key  | valueType | comment            |
| :--- | :-------- | :----------------- |
| code | int       | 信息类型           |
| time | int       | 消息发送时的时间戳 |

## 文本信息 json

**_Server <-> Client_**

格式如下

```json
{
  "code": 0,
  "time": 消息发送时的时间戳,
  "cid": 该信息的id,唯一, //传输给服务器时无此字段
  "sender": {
    "ocid":发送者ocid,
    "group_id":发送此信息的群聊(群聊情况下),
    "private_id":接收此信息的人(私聊情况下)
  },
  "msg": "文本信息"

}
```

| key        | valueType | comment                                               |
| :--------- | :-------- | :---------------------------------------------------- |
| code       | int       | 信息类型                                              |
| time       | int       | 消息发送时的时间戳                                    |
| cid        | int       | chat 的 ID，唯一 **_(注意：传输给服务器时无此字段)_** |
| sender     | int       | 发送者的相关数据                                      |
| ocid       | int       | 发送者的 ocid                                         |
| group_id   | int       | 当会话为群聊时，该值为群号                            |
| private_id | int       | 当会话为私聊时，该值为对方的 ocid                     |
| msg        | str       | 文本信息                                              |

**_code1,2,3 分别为还未制作的表情包(包括但不限于 gif)，图片发送，文件发送_**

## 注册信息 json

**_Server <- Client_**

格式如下

```json
{
  "code": 4,
  "time": 发送请求时的时间戳,
  "email": "注册使用的邮箱",
  "password": "注册密码(已加密)",
  "name": "昵称"
}
```

| key      | valueType | comment            |
| :------- | :-------- | :----------------- |
| code     | int       | 信息类型           |
| time     | int       | 发送请求时的时间戳 |
| email    | str       | 注册邮箱           |
| password | str       | 注册密码(已加密)   |
| name     | str       | 昵称               |

## 注册返回信息 json

**_Server -> Client_**

格式如下

```json
{
  "code": 5,
  "state": 返回码,
  "ocid": "注册账号的OC号"
}
```

| key   | valueType | comment            |
| :---- | :-------- | :----------------- |
| code  | int       | 信息类型           |
| state | int       | 服务端返回的状态码 |
| ocid  | int       | 该账号的 OC 号     |

| returnCode | comment    |
| :--------- | :--------- |
| 0          | 注册成功   |
| 1          | 服务器错误 |
| 2          | 邮箱重复   |

## 登录信息 json

**_Server <- Client_**

格式如下

```json
{
  "code": 6,
  "time": 发送请求时的时间戳,
  "login_type": 登陆方式,
  "account": "邮箱/OCID",
  "password": "密码"
}
```

| key        | valueType | comment                      |
| :--------- | :-------- | :--------------------------- |
| code       | int       | 信息类型                     |
| time       | int       | 发送请求时的时间戳           |
| login_type | int       | 0 为邮箱登录，1 为 ocid 登录 |
| account    | str       | 账号绑定的邮箱或 ocid        |
| password   | str       | 密码                         |

## 登录返回信息 json

**_Server -> Client_**

格式如下

```json
{
  "code": 7,
  "time": 时间戳,
  "state": 登录状态码,
  "ocid":该账号的ocid
}
```

| key   | valueType | comment            |
| :---- | :-------- | :----------------- |
| code  | int       | 信息类型           |
| time  | int       | 发送请求时的时间戳 |
| state | int       | 服务器返回的状态码 |
| ocid  | int       | 该账号的 OCID      |

| returnCode | comment          |
| :--------- | :--------------- |
| 0          | 登录成功         |
| 1          | 账号或密码不正确 |
| 2          | 服务器错误       |

## 新建会话请求信息 json

**_Server <- Client_**

```json
{
  "code": 8,
  "time": 时间戳,
  "members": [
    ocid1,
    ocid2,
    ...
  ]
}
```

| key     | valueType | comment            |
| :------ | :-------- | :----------------- |
| code    | int       | 信息类型           |
| time    | int       | 发送请求时的时间戳 |
| members | list      | 会话成员           |

## 新建会话返回信息 json

**_Server -> Client_**

```json
{
  "code": 9,
  "time": 时间戳,
  "success": true/false,
  "msg": "失败原因"
}
```

| key     | valueType | comment            |
| :------ | :-------- | :----------------- |
| code    | int       | 信息类型           |
| time    | int       | 发送请求时的时间戳 |
| success | bool      | 新建会话是否成功   |
| msg     | str       | 失败原因           |

## 获取账号信息 json

**_Server <- Client_**

```json
{
  "code": 10,
  "time": 时间戳,
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
| code           | int       | 信息类型             |
| time           | int       | 发送请求时的时间戳   |
| ocid           | int       | 该账号的 ocid        |
| request_values | list      | 需要服务端返回的信息 |

| request_value | comment               |
| :------------ | :-------------------- |
| ocid          | 该账号的 ocid         |
| nickname      | 昵称                  |
| status        | 该账号的状态          |
| avater        | 该账号头像的 url 链接 |
| time          | 该账号注册的时间戳    |

## 获取账号信息返回 json

**_Server -> Client_**

```json
{
  "code": 11,
  "time": 时间戳,
  "data":{
    "ocid": 该账号的OCID
    "nickname": 昵称,
    ...
  }
}
```

| key  | valueType | comment                                               |
| :--- | :-------- | :---------------------------------------------------- |
| code | int       | 信息类型                                              |
| time | int       | 发送请求时的时间戳                                    |
| data | json      | 账号信息,详情[见上`request_value`](#获取账号信息json) |
