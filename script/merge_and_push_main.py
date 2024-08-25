#!/usr/bin/env python3

from basic import msg_system

msg_system("git checkout main")
msg_system("git merge dev")
msg_system("git push")
msg_system("git checkout dev")
input("ok")
