#!/usr/bin/env python3

import os
import sys


def test_server(test_inst: list):
    os.chdir("server")
    # os.putenv("OURCHAT_CONFIG_FILE", "../local/ourchat.toml")
    os.putenv("RUST_BACKTRACE", "1")
    os.putenv("OURCHAT_ARGVS", " ".join(test_inst))
    code = os.system("cargo test -- --test-threads=1")
    print(code)
    return code


if __name__ == "__main__":
    if len(sys.argv) > 1:
        test_inst = sys.argv[1:]
    else:
        test_inst = []
    if test_server(test_inst) != 0:
        raise Exception("Server tests failed")
