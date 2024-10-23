#!/usr/bin/env python3

"""
Run migrations for both databases
"""

import os
from sys import argv

import basic


def main() -> int:
    sqlite_db_file = os.path.abspath("config/sqlite/ourchat.db")
    arg = " ".join(argv[1:])

    os.chdir("server")
    os.putenv("DATABASE_URL", f"sqlite://{sqlite_db_file}")
    basic.msg_system("sea migrate {}".format(arg))
    os.putenv("DATABASE_URL", "postgres://postgres:123456@localhost:5432/OurChat")
    basic.msg_system("sea migrate {}".format(arg))
    return 0


if __name__ == "__main__":
    main()
