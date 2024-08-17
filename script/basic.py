import os
import sys


def msg_system(cmd: str, msg=None):
    status = os.system(cmd)
    if status != 0:
        if msg is not None:
            print("ERROR:", msg)
        sys.exit(1)
