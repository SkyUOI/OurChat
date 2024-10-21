#!/usr/bin/env python3

import subprocess
import sys

from basic import msg_system

VERSION = "1.1.0"


def version_check():
    # get sea version
    sea_version = subprocess.getoutput("sea --version").split()[1]
    # check version
    if VERSION != sea_version:
        return False
    return True


def main():
    if not version_check():
        print("please install sea version {}".format(VERSION))
        sys.exit(1)
    msg_system(
        "sea generate entity -u mysql://root:123456@localhost:3306/OurChat -o server/src/entities/mysql"
    )
    msg_system(
        "sea generate entity -u sqlite://config/sqlite/ourchat.db -o server/src/entities/sqlite"
    )


if __name__ == "__main__":
    main()
