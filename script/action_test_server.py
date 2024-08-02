import init_valgrind_rust
import test_server
import os

init_valgrind_rust.init_valgrind()
os.system("apt update")
os.system("apt install valgrind -y")
if test_server.test_server() != 0:
    raise Exception("Server tests failed")
