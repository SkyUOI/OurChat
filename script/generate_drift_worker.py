#!/bin/python

import os, shutil
import basic

os.chdir("client")
os.remove("drift_worker.js")
os.remove("drift_worker.min.js")

# Debug build
shutil.rmtree("build/web", ignore_errors=True)
basic.msg_system(
    "dart run build_runner build --delete-conflicting-outputs -o web:build/web/"
)
shutil.copyfile("build/web/drift_worker.dart.js", "web/drift_worker.js")
shutil.rmtree("build/web", ignore_errors=True)

# Release build
shutil.rmtree("build/web", ignore_errors=True)
basic.msg_system(
    "dart run build_runner build --release --delete-conflicting-outputs -o web:build/web/"
)
shutil.copyfile("build/web/drift_worker.dart.js", "web/drift_worker.min.js")
shutil.rmtree("build/web", ignore_errors=True)
