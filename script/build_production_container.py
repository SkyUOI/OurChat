#!/usr/bin/env python3

from basic import msg_system
import argparse

parser = argparse.ArgumentParser(
    description="Build production container images for OurChat."
)
parser.add_argument(
    "--extension",
    type=str,
    default="latest",
    help="The extension tag for the images (default: latest).",
)
parser.add_argument(
    "--skip-base",
    default=True,
    action="store_true",
    help="Skip building the base images.",
)
parser.add_argument(
    "--push", default=False, action="store_true", help="Push the images to Docker Hub."
)
args = parser.parse_args()

extension = args.extension
skip_base = args.skip_base
args_pass = ""
if args.push:
    args_pass += "--push "

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
