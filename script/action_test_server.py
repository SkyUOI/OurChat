#!/usr/bin/env python3

import init_valgrind_rust
import test_server
from basic import msg_system

init_valgrind_rust.init_valgrind()
msg_system("apt update")
msg_system("apt install valgrind -y")
if test_server.test_server() != 0:
    raise Exception("Server tests failed")
