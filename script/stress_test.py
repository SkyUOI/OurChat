#!/usr/bin/env python3

import basic
import os
import sys

default_test_flag = "--release"
default_test_command = "cargo run --bin stress_test"


def test():
    if len(sys.argv) > 1:
        test_flag = sys.argv[1]
    else:
        test_flag = default_test_flag
    if len(sys.argv) > 2:
        test_command = sys.argv[2]
    else:
        test_command = default_test_command
    os.putenv("OURCHAT_LOG", "error")
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/gh_test/ourchat.toml"))
    basic.msg_system(f"{test_command} {test_flag}")


def main():
    test()


if __name__ == "__main__":
    main()
