import basic
import sys


def main():
    tag = "latest"
    skip_base = False
    if len(sys.argv) >= 2:
        tag = sys.argv[1]
        if len(sys.argv) >= 3:
            if sys.argv[2] == "skip_base":
                skip_base = True

    basic.msg_system(f"docker push skyuoi/ourchat:{tag}")
    basic.msg_system(f"docker push skyuoi/ourchat:{tag}-debian")

    if not skip_base:
        basic.msg_system("docker push skyuoi/ourchat:alpine-base")
        basic.msg_system("docker push skyuoi/ourchat:debian-base")

    basic.msg_system(f"docker push skyuoi/ourchat:{tag}-http")
    basic.msg_system(f"docker push skyuoi/ourchat:{tag}-http-debian")


if __name__ == "__main__":
    main()
