#!/usr/bin/env python3

from basic import msg_system
import argparse

parser = argparse.ArgumentParser(
    description="Build production container images for OurChat."
)
parser.add_argument(
    "--extension",
    action="append",
    help="The extension tag for the images (default: latest).",
)
parser.add_argument(
    "--skip-base",
    default=False,
    action="store_true",
    help="Skip building the base images.",
)
parser.add_argument(
    "--push", default=False, action="store_true", help="Push the images to Docker Hub."
)
parser.add_argument(
    "--sccache-bucket",
    default="",
    help="The OSS bucket name for sccache.",
)
parser.add_argument(
    "--sccache-endpoint",
    default="",
    help="The OSS endpoint for sccache.",
)
args = parser.parse_args()

enable_sccache = False
if args.sccache_bucket and args.sccache_endpoint:
    print("Enable sccache for building.")
    enable_sccache = True
elif args.sccache_bucket or args.sccache_endpoint:
    print("Both --sccache-bucket and --sccache-endpoint must be provided to enable sccache.")
    exit(1)

extension = args.extension or ["latest"]
skip_base = args.skip_base
args_pass = ""
if args.push:
    args_pass += "--push "

args_tags_alpine = ""
for i in extension:
    args_tags_alpine += f"-t skyuoi/ourchat:{i} "
args_tags_debian = ""
for i in extension:
    args_tags_debian += f"-t skyuoi/ourchat:{i}-debian "

if enable_sccache:
    args_pass += (
        f"--build-arg SCCACHE_OSS_BUCKET={args.sccache_bucket} "
        f"--build-arg SCCACHE_OSS_ENDPOINT={args.sccache_endpoint} "
        f"--build-arg SCCACHE_ENABLED=true "
    )

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
msg_system(f"docker buildx build -f docker/Dockerfile {args_pass} {args_tags_alpine} .")
# build debian image
msg_system(
    f"docker buildx build -f docker/Dockerfile.debian {args_pass} {args_tags_debian} ."
)
