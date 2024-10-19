"""
Run migrations for both sqlite and mysql
"""

from sys import argv
import os
import basic


def main() -> int:
    sqlite_db_file = os.path.abspath("config/sqlite/ourchat.db")

    os.chdir("server")
    os.putenv("DATABASE_URL", f"sqlite://{sqlite_db_file}")
    arg = " ".join(argv[1:])
    basic.msg_system("sea migrate {}".format(arg))
    os.putenv("DATABASE_URL", "mysql://root:123456@localhost:3306/OurChat")
    arg = " ".join(argv[1:])
    basic.msg_system("sea migrate {}".format(arg))


if __name__ == "__main__":
    main()
