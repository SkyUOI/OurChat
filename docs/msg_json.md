# 基本信息json
格式如下
```json
{
    "code" : 信息ID,
    "time" : 消息发送时的时间戳
    "data" : {
        # 根据信息所需的数据来定键值对
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code     |int          |信息ID           |
|data  |json         |信息相关数据      |
|time | int |消息发送时的时间戳|

# 文本信息json
格式如下
```json
{
    "code" : 0,
    "time" : 消息发送时的时间戳
    "data" : {
        "cid" : Chat的id,
        "sender_id" : 发送者ID,
        "msg" : "文本信息"
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|time | int |消息发送时的时间戳|
|cid      |int          |chat的ID，唯一    |
|sender_id|str          |发送者的ID，唯一  |
|data  |json         |信息相关数据      |
|msg    | str       |文本信息         |

***code1,2,3分别为还未制作的表情包(包括但不限于gif)，图片发送，文件发送***

# 注册信息json
格式如下
```json
{
    "code" : 4,
    "time" : 发送请求时的时间戳
    "data" : {
        "mail" : 注册使用的邮箱,
        "password" : 注册密码
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|time | int |发送请求时的时间戳|
|data  |json         |信息相关数据      |
|mail |   str       |注册邮箱         |
|password|str|注册密码|

# 注册返回信息json
格式如下
```json
{
    "code" : 5,
    "data" : {
        "state" : 返回码,
        "ocId" : 注册账号的OC号
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|data  |json         |信息相关数据      |
|state    | int      |服务端返回的状态码|
|ocId    | str      |该账号的OC号|

|returnCode|  comment   |
|:--------|:------------|
|0        |注册成功      |
|1        |服务器错误    |

# 登录信息json
格式如下
```json
{
    "code" : 6,
    "time" : 发送请求时的时间戳
    "data" : {
        "ocId" : 账号,
        "password" : 密码
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|time | int |发送请求时的时间戳|
|data  |json         |信息相关数据      |
|ocId   |str      |ocId  |
|password   |str      |密码  |

# 登录返回信息json
格式如下
```json
{
    "code" : 7,
    "data" : {
        "state" : 登录状态码
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|data  |json         |信息相关数据      |
|state    |int      |服务器返回的状态码  |

|returnCode|  comment   |
|:--------|:------------|
|0        |登录成功      |
|1        |账号或密码不正确 |

# 未接收信息json
在登录ourchat时服务端会将该账号没有接收的信息用此json发送给客户端

格式如下
```json
{
    "code" : 8,
    "data" : {
        "msg":[
            #单条信息的json
            '{"code":......}',
            '{"code":......}'
        ]
    }
}
```
| key     |valueType    |comment          |
|:--------|:------------|:----------------|
|code  |int          |信息ID           |
|data  |json         |信息相关数据      |
|msg   |list      |包含了多条未接受消息  |