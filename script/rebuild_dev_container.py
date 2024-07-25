import os

os.chdir("..")
os.system("docker build -t ourchat_develop:latest .")
