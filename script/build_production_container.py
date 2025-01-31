#!/usr/bin/env python3

from basic import msg_system
import os
import sys

extension = "latest"

if len(sys.argv) == 2:
    extension = sys.argv[1]

# build alpine image
msg_system(
    f"docker buildx build -f Dockerfile --target ourchat-server -t skyuoi/ourchat:{extension} ."
)
# build debian image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target ourchat-server -t skyuoi/ourchat:{extension}-debian ."
)
# build alpine base image
msg_system(
    f"docker buildx build -f docker/Dockerfile.alpine-base -t skyuoi/ourchat:alpine-base ."
)
# build debian base image
msg_system(
    f"docker buildx build -f docker/Dockerfile.debian-base -t skyuoi/ourchat:debian-base ."
)
# build alpine http image
msg_system(
    f"docker buildx build -f Dockerfile --target http-server -t skyuoi/ourchat:{extension}-http ."
)
# build debian http image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target http-server -t skyuoi/ourchat:{extension}-http-debian ."
)
