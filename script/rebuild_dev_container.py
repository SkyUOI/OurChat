#!/usr/bin/env python3

from basic import msg_system
import os
import sys

extension = "latest"
dockerfile = "Dockerfile.dev-aphine"

arg_len = len(sys.argv)

if arg_len >= 2:
    dockerfile = sys.argv[1]
    if arg_len >= 3:
        extension = sys.argv[2]

os.chdir("../docker")
msg_system(f"docker buildx build -f {dockerfile} -t ourchat_develop:{extension} .")
