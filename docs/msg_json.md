# 基本信息json
格式如下
```json
{
    "code" : 信息ID,
    "data" : {
        # 根据信息所需的数据来定键值对
    }
}
```
| key  | valueType | comment |
|:-----|:----------|:--------|
| code | int       | 信息ID    |
| data | json      | 信息相关数据  |

# 文本信息json
格式如下
```json
{
    "code" : 0,
    "data" : {
        "cid" : Chat的id,
        "sender_id" : 发送者ID,
        "msg" : "文本信息"
    }
}
```
| key       | valueType | comment    |
|:----------|:----------|:-----------|
| code      | int       | 信息ID       |
| cid       | int       | chat的ID，唯一 |
| sender_id | str       | 发送者的ID，唯一  |
| data      | json      | 信息相关数据     |
| msg       | str       | 文本信息       |


# 表情信息json
格式如下
```json
{
    "code" : 1,
    "data" : {
        "cid" : Chat的id,
        "sender_id" : 发送者ID,
        "emojiId" : 表情ID
    }
}
```
| key       | valueType | comment    |
|:----------|:----------|:-----------|
| code      | int       | 信息ID       |
| cid       | int       | chat的ID，唯一 |
| sender_id | str       | 发送者的ID，唯一  |
| data      | json      | 信息相关数据     |
| emojiId   | int       | 表情信息 见下表   |

| name | id  | icon |
|:-----|:----|:-----|
| 写    | 个   | der  |

# 图像信息json
格式如下
```json
{
    "code" : 2,
    "data" : {
        "cid" : Chat的id,
        "sender_id" : 发送者ID,
        "packages_num" : 数据包的数量,
        "size" : 图片文件大小
    }
}
```
| key          | valueType | comment              |
|:-------------|:----------|:---------------------|
| code         | int       | 信息ID                 |
| cid          | int       | chat的ID，唯一           |
| sender_id    | str       | 发送者的ID，唯一            |
| data         | json      | 信息相关数据               |
| packages_num | int       | 将图片拆成若干个包发送 此处填写包的数量 |
| size         | int       | 图片文件大小               |

# 文件信息json
格式如下
```json
{
    "code" : 3,
    "data" : {
        "cid" : Chat的id,
        "sender_id" : 发送者ID,
        "packages_num" : 数据包的数量,
        "size" : 文件大小
    }
}
```
| key          | valueType | comment              |
|:-------------|:----------|:---------------------|
| code         | int       | 信息ID                 |
| cid          | int       | chat的ID，唯一           |
| sender_id    | str       | 发送者的ID，唯一            |
| data         | json      | 信息相关数据               |
| packages_num | int       | 将文件拆成若干个包发送 此处填写包的数量 |
| size         | int       | 文件大小                 |

# 注册信息json
格式如下
```json
{
    "code" : 4,
    "data" : {
        "mail" : 注册使用的邮箱,
        "password" : 注册密码
    }
}
```
| key      | valueType | comment |
|:---------|:----------|:--------|
| code     | int       | 信息ID    |
| data     | json      | 信息相关数据  |
| mail     | str       | 注册邮箱    |
| password | str       | 注册密码    |

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
| key   | valueType | comment   |
|:------|:----------|:----------|
| code  | int       | 信息ID      |
| data  | json      | 信息相关数据    |
| state | int       | 服务端返回的状态码 |
| ocId  | str       | 该账号的OC号   |

| returnCode | comment |
|:-----------|:--------|
| 0          | 注册成功    |
| 1          | 服务器错误   |

# 登录信息json
格式如下
```json
{
    "code" : 6,
    "data" : {
        "ocId" : 账号,
        "password" : 密码
    }
}
```
| key      | valueType | comment |
|:---------|:----------|:--------|
| code     | int       | 信息ID    |
| data     | json      | 信息相关数据  |
| ocId     | str       | ocId    |
| password | str       | 密码      |

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
| key   | valueType | comment   |
|:------|:----------|:----------|
| code  | int       | 信息ID      |
| data  | json      | 信息相关数据    |
| state | int       | 服务器返回的状态码 |

| returnCode | comment  |
|:-----------|:---------|
| 0          | 登录成功     |
| 1          | 账号或密码不正确 |
