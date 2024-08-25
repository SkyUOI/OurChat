#!/usr/bin/env python3

import init_valgrind_rust
import test_server
import sys
from basic import msg_system

init_valgrind_rust.init_valgrind()
msg_system("apt update")
msg_system("apt install valgrind -y")
if len(sys.argv) > 1:
    test_inst = sys.argv[1:]
else:
    test_inst = []
if test_server.test_server(test_inst) != 0:
    raise Exception("Server tests failed")
