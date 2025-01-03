#!/usr/bin/env python3

import os
import sys

if os.name == "nt":
    # Notice this: windows cannot build load_balancer(by now)
    default_test_command = "cargo test --exclude load_balancer --workspace"
else:
    default_test_command = "cargo test"
default_test_cfg = "config/ourchat.toml"


def run_tests():
    if len(sys.argv) > 1:
        test_command = sys.argv[1]
    else:
        test_command = default_test_command
    print("Running tests...")
    test_process = os.system(test_command)
    if test_process != 0:
        raise Exception("Tests failed.")


def start_test():
    if len(sys.argv) > 2:
        cfg = sys.argv[2]
    else:
        cfg = default_test_cfg
    os.putenv("OURCHAT_CONFIG_FILE", os.path.abspath(cfg))
    run_tests()


def test_process() -> int:
    return_code = 0
    os.putenv(
        "OURCHAT_LOG",
        "trace,actix_web=off,actix_server=off,mio=off,h2=off,tonic=off,tower=off,tokio::task::waker=off,runtime::resource=off",
    )
    # Run tests
    start_test()
    return return_code


def main():
    sys.exit(test_process())


if __name__ == "__main__":
    main()
