#!/usr/bin/env python3

from basic import msg_system
import os
import sys

extension = "latest"

if len(sys.argv) == 2:
    extension = sys.argv[1]

# build aphine image
msg_system(
    f"docker buildx build -f Dockerfile --target ourchat-server -t skyuoi/ourchat:{extension} ."
)
# build debian image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target ourchat-server -t skyuoi/ourchat:{extension}-debian ."
)
# build aphine base image
msg_system(
    f"docker buildx build -f docker/Dockerfile.aphine-base -t skyuoi/ourchat:aphine-base ."
)
# build debian base image
msg_system(
    f"docker buildx build -f docker/Dockerfile.debian-base -t skyuoi/ourchat:debian-base ."
)
# build aphine http image
msg_system(
    f"docker buildx build -f Dockerfile --target http-server -t skyuoi/ourchat:{extension}-http ."
)
# build debian http image
msg_system(
    f"docker buildx build -f Dockerfile.debian --target http-server -t skyuoi/ourchat:{extension}-http-debian ."
)
