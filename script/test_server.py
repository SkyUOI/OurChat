import os


def test_server():
    os.chdir("server")
    # os.putenv("OURCHAT_CONFIG_FILE", "../local/ourchat.toml")
    code = os.system("cargo test -- --test-threads=1")
    print(code)
    return code


if __name__ == "__main__":
    if test_server() != 0:
        raise Exception("Server tests failed")
