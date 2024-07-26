import os

os.chdir("..")
_ = os.system("docker build -f Dockerfile.dev -t ourchat_develop:latest .")
