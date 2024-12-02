#!/usr/bin/env python3

import sys

from basic import VERSION, msg_system, version_check


def main():
    if not version_check():
        print("please install sea version {}".format(VERSION))
        sys.exit(1)
    msg_system(
        "sea generate entity -u \
        postgres://postgres:123456@localhost:5432/OurChat -o server/src/entities"
    )


if __name__ == "__main__":
    main()
