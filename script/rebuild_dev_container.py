import os

_ = os.system("docker buildx build -f Dockerfile.dev -t ourchat_develop:latest .")
