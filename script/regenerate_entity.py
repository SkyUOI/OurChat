#!/usr/bin/env python3

from basic import msg_system

msg_system(
    "sea generate entity -u mysql://root:123456@localhost:3306/OurChat -o server/src/entities/mysql"
)
msg_system(
    "sea generate entity -u sqlite://server/ourchat.db -o server/src/entities/sqlite"
)
