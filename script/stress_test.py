import basic
import os
import sys

default_test_flag = "--release"
test_command = "cargo run --bin stress_test"


def test():
    if len(sys.argv) > 1:
        test_flag = sys.argv[1]
    else:
        test_flag = default_test_flag
    os.putenv("OURCHAT_LOG", "error")
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/gh_test/ourchat.toml"))
    basic.msg_system(f"{test_command} {test_flag}")


def main():
    test()


if __name__ == "__main__":
    main()
