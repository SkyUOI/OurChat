#!/usr/bin/env python3

from basic import msg_system
import os
import sys

extension = "latest"

if len(sys.argv) == 2:
    extension = sys.argv[1]

os.chdir("../docker")
# build aphine image
msg_system(f"docker buildx build -f Dockerfile -t skyuoi/ourchat:aphine-{extension} .")
# build debian image
msg_system(f"docker buildx build -f Dockerfile -t skyuoi/ourchat:{extension} .")
