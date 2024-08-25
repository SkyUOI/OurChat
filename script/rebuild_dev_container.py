#!/usr/bin/env python3

from basic import msg_system

msg_system("docker buildx build -f Dockerfile.dev -t ourchat_develop:latest .")
