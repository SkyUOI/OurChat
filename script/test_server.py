import os
import subprocess
import json
import sys

import basic

test_command = "cargo test "


def start_server(exec: str, args: list):
    # 启动服务器
    print("Starting the server...")
    process = subprocess.Popen([exec] + args, env={"RUST_LOG": "debug"})

    print("Waiting for the server to start...")
    return process


def run_tests(args: list):
    print("Running tests...")
    test_process = subprocess.run(
        test_command.split() + args, stdout=subprocess.PIPE, stderr=sys.stderr
    )

    print("Test output:")
    print(test_process.stdout.decode())

    if test_process.returncode == 0:
        print("Tests passed.")
    else:
        print("Tests failed.")


class Config:
    def __init__(self, json: dict):
        self.server_args = json["server_args"]
        self.test_args = json["test_args"]
        self.server_exec = json["server_exec"]
        self.cmd_before_run = json["cmd_before_run"]


def read_cfg(path) -> Config:
    with open(path, "r") as f:
        return Config(json.load(f))


def main():
    if len(sys.argv) > 1:
        cfg = sys.argv[1]
    else:
        cfg = os.path.join(os.path.dirname(__file__), "server_test.json")
    cfg = read_cfg(cfg)
    if cfg.cmd_before_run != "":
        basic.msg_system(cfg.cmd_before_run)
    server_process = start_server(cfg.server_exec, cfg.server_args)

    try:
        # 运行测试
        run_tests(cfg.test_args)
    finally:
        # 终止服务器进程
        print("Terminating the server...")
        server_process.terminate()
        server_process.wait()
        print("Server terminated.")


if __name__ == "__main__":
    main()
