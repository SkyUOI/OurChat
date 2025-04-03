#!/usr/bin/env python3

import sys
from shutil import rmtree

import basic


def main():
    input(
        "This is a dangerous operation, which will delete your database. Press enter to continue or ctrl+c to cancel."
    )
    if len(sys.argv) < 2:
        compose_file = "docker/compose.yml"
    else:
        compose_file = sys.argv[1]
    basic.msg_system(f"docker compose -f {compose_file} down")
    basic.msg_system("docker pull skyuoi/ourchat:latest")
    basic.msg_system("docker pull skyuoi/ourchat:latest-http")
    basic.msg_system("docker pull skyuoi/ourchat:nightly")
    basic.msg_system("docker pull skyuoi/ourchat:nightly-http")
    rmtree("docker/data/postgres-data")
    basic.msg_system(f"docker compose -f {compose_file} up -d")


if __name__ == "__main__":
    main()
