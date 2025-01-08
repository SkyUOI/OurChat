#!/usr/bin/env python3

import os

import init_valgrind_rust

init_valgrind_rust.init_valgrind()
os.putenv("OURCHAT_TEST_CONFIG_DIR", "config/")
os.system("cargo test")
