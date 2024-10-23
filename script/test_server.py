import os
import sys

default_test_command = "cargo test"


def run_tests():
    if len(sys.argv) > 2:
        test_command = sys.argv[2]
    else:
        test_command = default_test_command
    print("Running tests...")
    test_process = os.system(test_command)
    if test_process != 0:
        raise Exception("Tests failed.")


def sqlite_test():
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/sqlite/ourchat.toml"))
    run_tests()


def postgres_test():
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/postgres/ourchat.toml"))
    run_tests()


def test_process() -> int:
    return_code = 0
    # Run tests
    test_suite = sys.argv[1]
    if test_suite == "sqlite":
        sqlite_test()
    elif test_suite == "postgres":
        postgres_test()
    else:
        sqlite_test()
        postgres_test()
    return return_code


def main():
    sys.exit(test_process())


if __name__ == "__main__":
    main()
