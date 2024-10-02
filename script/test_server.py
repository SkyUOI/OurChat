import os
import sys


test_command = "cargo test --bin server"


def run_tests():
    print("Running tests...")
    test_process = os.system(test_command)
    if test_process != 0:
        raise Exception("Tests failed.")


def sqlite_test():
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/sqlite/ourchat.toml"))
    run_tests()


def mysql_test():
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/mysql/ourchat.toml"))
    run_tests()


def test_process() -> int:
    return_code = 0
    # Run tests
    test_suite = sys.argv[1]
    if test_suite == "sqlite":
        sqlite_test()
    elif test_suite == "mysql":
        mysql_test()
    else:
        sqlite_test()
        mysql_test()
    return return_code


def main():
    sys.exit(test_process())


if __name__ == "__main__":
    main()
