#!/usr/bin/env python3
"""Run OurChat client integration tests against live server instances.

Automates the "bring up servers, run tests, tear down" workflow for the Flutter
client integration tests (which require real servers):

    1. Preflight: tool availability, dependency ports (db/redis/mq), free ports.
    2. Build the server binary (cargo build --bin server).
    3. Generate isolated configs for two server instances (A:<port-a>,
       B:<port-b> with its own rabbitmq vhost).
    4. Start both servers and wait until they answer.
    5. Run the requested integration tests one file at a time.
    6. Clean up: kill the servers, remove temp files. Docker is NOT managed
       here — start the dev dependencies yourself first:

           docker compose -f docker/compose.devenv.yml up -d db redis mq

Usage:
    python script/run_integration_tests.py [--test integration_test/xxx.dart]
                                          [--port-a 7777] [--port-b 7778]
                                          [--server-path /path/to/server]
                                          [--keep-server] [--debug]
"""

import argparse
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SERVER_DIR = os.path.join(ROOT, "server")
DOCKER_CONFIG_DIR = os.path.join(ROOT, "docker", "config")
RESOURCE_DIR = os.path.join(ROOT, "resource")
WEB_PANEL_DIST = os.path.join(ROOT, "server", "web-panel", "dist")
CLIENT_DIR = os.path.join(ROOT, "client")

DEFAULT_TESTS = [
    "multi_server_test.dart",
    "quote_flow_test.dart",
    "quote_client_logic_test.dart",
    "e2ee_quote_test.dart",
    "quote_ui_e2e_test.dart",
]
VHOST_B = "oc_e2e_b"
DEPENDENCY_PORTS = (5432, 6379, 5672)


def fail(msg):
    print(f"ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def port_open(port, host="localhost", timeout=1.0):
    """Try all resolved addresses (IPv4 + IPv6) for [host]:[port]."""
    try:
        addrs = socket.getaddrinfo(host, port, type=socket.SOCK_STREAM)
    except socket.gaierror:
        return False
    for af, _, _, _, sa in addrs:
        with socket.socket(af, socket.SOCK_STREAM) as s:
            s.settimeout(timeout)
            try:
                s.connect(sa)
                return True
            except OSError:
                continue
    return False


def preflight(args):
    for tool in ("cargo", "flutter", "docker"):
        if shutil.which(tool) is None:
            fail(f"missing required tool: {tool}")

    for port in DEPENDENCY_PORTS:
        if not port_open(port):
            print(
                "ERROR: dependency port %d is not reachable.\n"
                "Start the dev environment first, for example:\n"
                "    docker compose -f docker/compose.devenv.yml up -d db redis mq"
                % port,
                file=sys.stderr,
            )
            sys.exit(1)

    for port in (args.port_a, args.port_b):
        if port_open(port):
            fail(f"port {port} is already in use (an old server may be running)")


def build_server(args):
    if args.server_path:
        if not os.path.isfile(args.server_path):
            fail(f"--server-path {args.server_path} does not exist")
        return args.server_path
    print("==> building server (cargo build --bin server)")
    subprocess.run(
        ["cargo", "build", "--bin", "server"],
        cwd=ROOT,
        check=True,
    )
    return os.path.join(ROOT, "target", "debug", "server")


def patch(d, name, old, new):
    p = os.path.join(d, name)
    s = open(p).read()
    if old in s:
        open(p, "w").write(s.replace(old, new))
    else:
        print(f"WARN: pattern {old!r} not found in {name}; skipping")


def copy_and_patch_configs(tmp, label, vhost):
    """Copy docker/config sub-configs into tmp/<label> and adapt them so a
    host-machine server process can connect to localhost-backed docker services.
    """
    d = os.path.join(tmp, label)
    os.makedirs(d, exist_ok=True)
    for name in (
        "database.toml",
        "redis.toml",
        "user_setting.toml",
        "http.toml",
        "email.toml",
        "rabbitmq.toml",
    ):
        shutil.copy(os.path.join(DOCKER_CONFIG_DIR, name), os.path.join(d, name))

    patch(d, "database.toml", 'host = "db"', 'host = "localhost"')
    patch(d, "redis.toml", 'host = "redis"', 'host = "localhost"')
    patch(d, "rabbitmq.toml", 'host = "mq"', 'host = "localhost"')
    if vhost != "/":
        patch(d, "rabbitmq.toml", 'vhost = "/"', f'vhost = "{vhost}"')

    http = os.path.join(d, "http.toml")
    s = open(http).read()
    s = s.replace('ip = "0.0.0.0"', 'ip = "localhost"')
    s = s.replace(
        'logo_path = "/app/resource/logo.png"',
        f'logo_path = "{os.path.join(RESOURCE_DIR, "logo.png")}"',
    )
    s = s.replace(
        'default_avatar_path = "/app/resource/logo.png"',
        f'default_avatar_path = "{os.path.join(RESOURCE_DIR, "logo.png")}"',
    )
    s = s.replace(
        'verification_html_template_path = "/app/resource/email.html"',
        f'verification_html_template_path = "{os.path.join(RESOURCE_DIR, "email.html")}"',
    )
    s = s.replace(
        'dist_path = "/app/resource/web-panel"',
        f'dist_path = "{WEB_PANEL_DIST}"',
    )
    open(http, "w").write(s)

    def sub(name):
        return os.path.join(d, name)

    ourchat = (
        f'inherit = "{os.path.join(DOCKER_CONFIG_DIR, "ourchat.toml")}"\n'
        f'db_cfg = "{sub("database.toml")}"\n'
        f'redis_cfg = "{sub("redis.toml")}"\n'
        f'user_setting = "{sub("user_setting.toml")}"\n'
        f'http_cfg = "{sub("http.toml")}"\n'
        f'rabbitmq_cfg = "{sub("rabbitmq.toml")}"\n'
        f'files_storage_path = "{os.path.join(ROOT, "files_storage")}"\n'
    )
    open(os.path.join(d, "ourchat.toml"), "w").write(ourchat)
    return d


def find_mq_container():
    out = subprocess.getoutput("docker ps --format '{{.Names}}\t{{.Ports}}'")
    for line in out.splitlines():
        if "5672" in line:
            return line.split("\t")[0]
    return None


def ensure_vhost(container, vhost):
    if not container:
        print(
            "WARN: cannot locate the rabbitmq container; server B vhost setup skipped"
        )
        return
    subprocess.run(
        ["docker", "exec", container, "rabbitmqctl", "add_vhost", vhost],
        capture_output=True,
    )
    subprocess.run(
        [
            "docker",
            "exec",
            container,
            "rabbitmqctl",
            "set_permissions",
            "-p",
            vhost,
            "guest",
            ".*",
            ".*",
            ".*",
        ],
        capture_output=True,
    )
    print(f"==> ensured rabbitmq vhost {vhost} on {container}")


def start_server(bin_path, cfg_dir, port, log_path):
    info = os.path.join(cfg_dir, "server_info.json")
    logf = open(log_path, "w")
    proc = subprocess.Popen(
        [
            bin_path,
            "-c",
            os.path.join(cfg_dir, "ourchat.toml"),
            "-p",
            str(port),
            "--server-info",
            info,
        ],
        cwd=ROOT,
        stdout=logf,
        stderr=subprocess.STDOUT,
    )
    return proc


def wait_server(port, log_path, proc, timeout=60):
    deadline = time.time() + timeout
    while time.time() < deadline:
        if proc.poll() is not None:
            print(f"server on port {port} exited early (code {proc.returncode})")
            break
        if port_open(port):
            # Give gRPC services a moment to finish initialization (db/mq).
            time.sleep(3)
            return True
        time.sleep(0.5)
    print(f"server on port {port} did not become ready; last log lines:")
    try:
        with open(log_path) as f:
            for line in f.readlines()[-15:]:
                print("   ", line.rstrip())
    except OSError:
        pass
    return False


def run_test(test_file):
    # Accept either "multi_server_test.dart" or "integration_test/multi_server_test.dart".
    if test_file.startswith("integration_test/"):
        test_file = test_file[len("integration_test/") :]
    print(f"==> running {test_file}")
    cmd = [
        "flutter",
        "test",
        os.path.join("integration_test", test_file),
        "-d",
        "linux",
    ]
    proc = subprocess.Popen(
        cmd,
        cwd=CLIENT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    lines = []
    # Stream stdout line-by-line for live progress (defensive: proc.stdout may
    # be None if PIPE is not used in some environments).
    if proc.stdout is not None:
        for line in proc.stdout:
            print(line, end="")
            lines.append(line)
    rc = proc.wait()
    skipped = any("All tests skipped" in l for l in lines)
    return rc, skipped


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--test",
        dest="tests",
        action="append",
        help="integration test file to run (repeatable; default: all)",
    )
    ap.add_argument("--port-a", type=int, default=7777)
    ap.add_argument("--port-b", type=int, default=7778)
    ap.add_argument(
        "--server-path", default=None, help="path to a prebuilt server binary"
    )
    ap.add_argument(
        "--keep-server",
        action="store_true",
        help="leave the servers running after tests",
    )
    ap.add_argument(
        "--debug",
        action="store_true",
        help="keep temp files and servers, print server paths",
    )
    args = ap.parse_args()

    preflight(args)
    server_bin = build_server(args)

    tmp = tempfile.mkdtemp(prefix="ourchat_e2e_")
    cfg_a = copy_and_patch_configs(tmp, "a", "/")
    cfg_b = copy_and_patch_configs(tmp, "b", VHOST_B)
    ensure_vhost(find_mq_container(), VHOST_B)

    log_a = os.path.join(tmp, "a.log")
    log_b = os.path.join(tmp, "b.log")
    procs = [
        start_server(server_bin, cfg_a, args.port_a, log_a),
        start_server(server_bin, cfg_b, args.port_b, log_b),
    ]
    print(f"==> server A: port {args.port_a}  (log {log_a})")
    print(f"==> server B: port {args.port_b}  (log {log_b})")

    exit_code = 0
    try:
        ok_a = wait_server(args.port_a, log_a, procs[0])
        ok_b = wait_server(args.port_b, log_b, procs[1])
        if not ok_a or not ok_b:
            fail("server(s) failed to become ready; see logs above")

        tests = args.tests if args.tests else DEFAULT_TESTS
        results = {}
        for t in tests:
            rc, skipped = run_test(t)
            results[t] = (rc, skipped)

        print("\n=== integration test results ===")
        for t, (rc, skipped) in results.items():
            status = "SKIP" if skipped else ("PASS" if rc == 0 else "FAIL")
            print(f"  {status:4}  {t}")
            if rc != 0:
                exit_code = 1
        if exit_code != 0:
            print(
                "test failed; temporary files were cleaned up "
                "(re-run with --debug to preserve logs)"
            )
    finally:
        if not args.keep_server and not args.debug:
            for p in procs:
                p.terminate()
            for p in procs:
                try:
                    p.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    p.kill()
        if not args.debug:
            shutil.rmtree(tmp, ignore_errors=True)

    if exit_code == 0:
        print("All integration tests passed.")
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
