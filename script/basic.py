import os
import subprocess
import sys


def msg_system(cmd: str, msg=None):
    status = os.system(cmd)
    if status != 0:
        if msg is not None:
            print("ERROR:", msg)
        sys.exit(1)


VERSION = "1.1.12"


def version_check():
    # get the sea version
    sea_version = subprocess.getoutput("sea --version").split()[1]
    # check the version
    if VERSION != sea_version:
        return False
    return True
