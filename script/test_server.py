import os

os.chdir("../server")
os.system("cargo test -- --test-threads=1")
