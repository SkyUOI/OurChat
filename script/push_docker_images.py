import basic


def main():
    basic.msg_system("docker push skyuoi/ourchat:latest")
    basic.msg_system("docker push skyuoi/ourchat:latest-debian")
    basic.msg_system("docker push skyuoi/ourchat:latest-debian")
    basic.msg_system("docker push skyuoi/ourchat:aphine-test")
    basic.msg_system("docker push skyuoi/ourchat:debian-test")

    basic.msg_system("docker push skyuoi/ourchat:latest-http")
    basic.msg_system("docker push skyuoi/ourchat:latest-http-debian")


if __name__ == "__main__":
    main()
