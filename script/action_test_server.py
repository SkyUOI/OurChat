#!/usr/bin/env python3

import init_valgrind_rust
import os

init_valgrind_rust.init_valgrind()
os.chdir("server")
if os.system("cargo test") != 0:
    raise Exception("Server tests failed")
