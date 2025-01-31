import basic


def main():
    basic.msg_system("docker push skyuoi/ourchat:latest")
    basic.msg_system("docker push skyuoi/ourchat:latest-debian")
    basic.msg_system("docker push skyuoi/ourchat:alpine-base")
    basic.msg_system("docker push skyuoi/ourchat:debian-base")

    basic.msg_system("docker push skyuoi/ourchat:latest-http")
    basic.msg_system("docker push skyuoi/ourchat:latest-http-debian")


if __name__ == "__main__":
    main()
