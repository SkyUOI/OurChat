# OurChat Message Passing Format

## Table of Contents

- [User Information](#user-information)
- [Registration Information](#registration-information)
- [Registration Return Information](#registration-return-information)
- [Login Information](#login-information)
- [Login Return Information](#login-return-information)
- [New Session Request Information](#new-session-request-information)
- [New Session Return Information](#new-session-return-information)
- [Get Account Information](#get-account-information)
- [Get Account Information Return Information](#get-account-information-return-information)
- [Get Server Status](#get-server-status)
- [Initiate Verification](#initiate-verification)
- [Generate Verification Code](#generate-verification-code)
- [Verification Status](#verification-status)
- [Logout](#logout)
- [Logout Return Information](#logout-return-information)
- [Error Information](#error-information)

## User Information

**_Server <-> Client_**

```json
{
  "code": 0,
  "time": Timestamp of the message sent,
  "msg_id": The unique ID of the message, // This field is not present when transmitting to the server
  "sender": {
    "ocid": The ocid of the sender,
    "session_id": The session ID of the message sent
  },
  "msg": [
    {
      "type": User message type,
      ... // Relevant data
    },
    ...
  ]
}
```

| Key        | ValueType | Comment                                                                                                      |
| :--------- | :-------- | :----------------------------------------------------------------------------------------------------------- |
| code       | Number    | Message type                                                                                                 |
| time       | Number    | Timestamp of the message sent                                                                                |
| msg_id     | Number    | The ID of the message, unique **_(Please note: This field is not present when transmitting to the server)_** |
| sender     | Object    | Relevant data of the sender                                                                                  |
| ocid       | String    | The ocid of the sender                                                                                       |
| session_id | Number    | The session ID of the sender                                                                                 |
| msg        | Array     | Message list                                                                                                 |
| type       | Number    | User message type, details see [User Message Passing Format](user_msg_json.md)                               |

## Registration Information

**_Server <- Client_**

```json
{
  "code": 4,
  "email": "Email used for registration",
  "password": "Encrypted registration password",
  "name": "Nickname"
}
```

| Key      | ValueType | Comment                         |
| :------- | :-------- | :------------------------------ |
| code     | Number    | Message type                    |
| email    | String    | Registration email              |
| password | String    | Encrypted registration password |
| name     | String    | Nickname                        |

## Registration Return Information

**_Server -> Client_**

```json
{
  "code": 5,
  "status": Return code,
  "ocid": "OC number of the registered account"
}
```

| Key    | ValueType | Comment                   |
| :----- | :-------- | :------------------------ |
| code   | Number    | Message type              |
| status | Number    | Server return status code |
| ocid   | Number    | OC number of the account  |

| ReturnCode | Comment                 |
| :--------- | :---------------------- |
| 0          | Registration successful |
| 1          | Server error            |
| 2          | Email already in use    |

## Login Information

**_Server <- Client_**

```json
{
  "code": 6,
  "login_type": Login method,
  "account": "Email/OCID",
  "password": "Password"
}
```

| Key        | ValueType | Comment                             |
| :--------- | :-------- | :---------------------------------- |
| code       | Number    | Message type                        |
| login_type | Number    | 0 for email login, 1 for ocid login |
| account    | String    | Email or ocid bound to the account  |
| password   | String    | Password                            |

## Login Return Information

**_Server -> Client_**

```json
{
  "code": 7,
  "status": Login status code,
  "ocid": The ocid of the account
}
```

| Key    | ValueType | Comment                   |
| :----- | :-------- | :------------------------ |
| code   | Number    | Message type              |
| status | Number    | Server return status code |
| ocid   | Number    | The ocid of the account   |

| Status | Comment                       |
| :----- | :---------------------------- |
| 0      | Login successful              |
| 1      | Incorrect account or password |
| 2      | Server error                  |

## New Session Request Information

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

| Key     | ValueType | Comment         |
| :------ | :-------- | :-------------- |
| code    | Number    | Message type    |
| members | Array     | Session members |

## New Session Return Information

**_Server -> Client_**

```json
{
  "code": 9,
  "status": Session status code,
  "session_id": Session id // Only present if creation is successful
}
```

| Key        | ValueType | Comment             |
| :--------- | :-------- | :------------------ |
| code       | Number    | Message type        |
| status     | Number    | Session status code |
| session_id | Number    | Session id          |

| Status | Comment                            |
| :----- | :--------------------------------- |
| 0      | Creation successful                |
| 1      | Server error                       |
| 2      | Reached the session creation limit |

## Get Account Information

**_Server <- Client_**

```json
{
  "code": 10,
  "ocid": The ocid of the account,
  "request_values":[
    "ocid",
    "nickname",
    ...
  ]
}
```

| Key            | ValueType | Comment                                         |
| :------------- | :-------- | :---------------------------------------------- |
| code           | Number    | Message type                                    |
| ocid           | Number    | The ocid of the account                         |
| request_values | Array     | Information needed to be returned by the server |

| RequestValue | Comment                               |
| :----------- | :------------------------------------ |
| ocid         | The ocid of the account               |
| nickname     | Nickname                              |
| status       | The status of the account             |
| avatar       | URL link of the account's avatar      |
| time         | Timestamp of the account registration |

## Get Account Information Return Information

**_Server -> Client_**

```json
{
  "code": 11,
  "data":{
    "ocid": The ocid of the account,
    "nickname": Nickname,
    ...
  }
}
```

| Key  | ValueType | Comment                                                                            |
| :--- | :-------- | :--------------------------------------------------------------------------------- |
| code | Number    | Message type                                                                       |
| data | Object    | Account information, details [see above `request_value`](#get-account-information) |

## Get Server Status

**_Server <-> Client_**

```json
{
  "code": 12,
  "status": Server status code, // This field is not present when transmitting to the server
}
```

| Key    | ValueType | Comment            |
| :----- | :-------- | :----------------- |
| code   | Number    | Message type       |
| status | Number    | Server status code |

| Status | Comment           |
| :----- | :---------------- |
| 0      | Running normally  |
| 1      | Under maintenance |

## Initiate Verification

**_Server -> Client_**

```json
{
  "code": 13
}
```

| Key  | ValueType | Comment      |
| :--- | :-------- | :----------- |
| code | Number    | Message type |

## Generate Verification Code

**_Server <- Client_**

```json
{
  "code": 14
}
```

## Verification Status

**_Server -> Client_**

```json
{
  "code": 15,
  "status": Verification status code
}
```

| Key    | ValueType | Comment                  |
| :----- | :-------- | :----------------------- |
| code   | Number    | Message type             |
| status | Number    | Verification status code |

| Status | Comment                |
| :----- | :--------------------- |
| 0      | Verification passed    |
| 1      | Verification failed    |
| 2      | Verification timed out |

## Logout

**_Server<-Client_**

```json
{
  "code": 16
}
```

**_Warning: This logout refers to the deletion of the account, do not misuse the interface_**

## Logout Return Information

**_Server -> Client_**

```json
{
  "code": 17,
  "status": Logout status code
}
```

| Key    | ValueType | Comment            |
| :----- | :-------- | :----------------- |
| code   | Number    | Message type       |
| status | Number    | Logout status code |

| Status | Comment           |
| :----- | :---------------- |
| 0      | Logout successful |
| 1      | Logout failed     |

## Error Information

**_Server -> Client_**

```json
{
  "code": 18,
  "details": "Error details"
}
```

| Key     | ValueType | Comment       |
| :------ | :-------- | :------------ |
| code    | Number    | Message type  |
| details | String    | Error message |
