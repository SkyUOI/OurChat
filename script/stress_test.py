import basic


def mkcmd(arg):
    return f"""ghz --insecure\
                     --proto ./message/service.proto\
                    {arg}\
                     --async\
                     localhost:7777"""


def test():
    basic.msg_system(mkcmd("--call ourchat.BasicService.timestamp"))
    basic.msg_system(mkcmd("--call ourchat.BasicService.get_server_info"))


def main():
    test()


if __name__ == "__main__":
    main()
