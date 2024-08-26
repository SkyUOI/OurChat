#!/usr/bin/env python3

import sys

import init_valgrind_rust
import test_server

init_valgrind_rust.init_valgrind()
if len(sys.argv) > 1:
    test_inst = sys.argv[1:]
else:
    test_inst = []
if test_server.test_server(test_inst) != 0:
    raise Exception("Server tests failed")
