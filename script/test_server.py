import os
import sys


test_command = "cargo test "


def run_tests():
    print("Running tests...")
    test_process = os.system(test_command)
    if test_process != 0:
        raise Exception("Tests failed.")


def test_process() -> int:
    return_code = 0
    # Run tests
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/sqlite/ourchat.toml"))
    run_tests()
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath("config/mysql/ourchat.toml"))
    run_tests()
    return return_code


def main():
    sys.exit(test_process())


if __name__ == "__main__":
    main()
