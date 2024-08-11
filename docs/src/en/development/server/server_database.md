# Server Database Design

Note: For convenience and accuracy of documentation, please refer directly to the [ORM entities](https://github.com/SkyUOI/OurChat/tree/main/server/src/entities) for specific data types.

## User Table (user)

| Column Name | Purpose                                             |
| :---------- | :-------------------------------------------------- |
| id          | Primary key, unique identifier (using Snowflake ID) |
| ocid        | Unique ID displayed to users (using a randomly generated 10-character string) |
| passwd      | User's password, encrypted with SHA-256, irreversible |
| name        | Nickname                                             |
| email       | Email                                                |
| date        | Timestamp                                            |

## Friend Table (friend)

| Column Name | Purpose                             |
| :---------- | :---------------------------------- |
| user_id    | User                                 |
| friend_id  | The user's friend                   |
| name        | Alias name set by the user for the friend |

## Chat Table (chat)

| Column Name | Purpose                                       |
| :---------- | :-------------------------------------------- |
| group_id    | Group chat                                    |
| user_id     | Marks the user in the group                  |
| name        | Marks the nickname of the user in the group  |
| group_name  | User's alias for the group chat, default is empty |

## Group Chat Table (groupchat)

| Column Name  | Purpose      |
| :----------- | :----------- |
| group_id     | Group chat ID |
| group_name   | Group chat name |

## User Chat Message Table (user_chat_msg)

| Column Name   | Purpose    |
| :------------ | :--------- |
| user_id       | User's ID  |
| chat_msg_id   | Message's ID |

## Chat Message Table (user_chat_id)

| Column Name   | Purpose          |
| :------------ | :--------------- |
| char_msg_id   | Chat message ID   |
| msg_type      | Type of the message |
| msg_data      | Data of the message |
| group_id      | Which group it belongs to |
| sender_id     | Sender's ID      |
