import os

os.chdir("..")
os.system("docker build -f Dockerfile.dev -t ourchat_develop:latest .")
