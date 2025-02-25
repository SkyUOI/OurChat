#!/usr/bin/env python3

from basic import msg_system
import sys

extension = "latest"
skip_base = False
args_pass = " ".join(sys.argv[3:])

if len(sys.argv) >= 2:
    extension = sys.argv[1]
    if len(sys.argv) >= 3:
        if sys.argv[2] == "skip_base":
            skip_base = True

if not skip_base:
    # build alpine base image
    msg_system(
        f"docker buildx build -f docker/Dockerfile.alpine-base {args_pass} -t skyuoi/ourchat:alpine-base ."
    )
    # build debian base image
    msg_system(
        f"docker buildx build -f docker/Dockerfile.debian-base {args_pass} -t skyuoi/ourchat:debian-base ."
    )
# build alpine image
msg_system(
    f"docker buildx build -f Dockerfile --target ourchat-server {args_pass} -t skyuoi/ourchat:{extension} ."
)
# build debian image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target ourchat-server {args_pass} -t skyuoi/ourchat:{extension}-debian ."
)
# build alpine http image
msg_system(
    f"docker buildx build -f Dockerfile --target http-server {args_pass} -t skyuoi/ourchat:{extension}-http ."
)
# build debian http image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target http-server {args_pass} -t skyuoi/ourchat:{extension}-http-debian ."
)
