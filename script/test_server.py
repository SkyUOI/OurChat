import os


def test_server():
    os.chdir("../server")
    return os.system("cargo test -- --test-threads=1")


if __name__ == "__main__":
    exit(test_server())
