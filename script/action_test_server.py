#!/usr/bin/env python3

import init_valgrind_rust
import os
import test_server

init_valgrind_rust.init_valgrind()
code = test_server.test_process()
if code != 0:
    os._exit(code)
