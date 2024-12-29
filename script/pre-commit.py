import os


def main():
    # format rust
    if os.system("cargo fmt"):
        print("It seems rust is not installed.Ignored")
    # format protobuf files
    if os.system("buf format --write"):
        print("It seems buf is not installed installed.Ignored")
    # format python scripts
    if os.system("ruff format"):
        print("It seems ruff is not installed.Ignored")
    # format dart
    os.chdir("client/ourchat/")
    if os.system("dart format lib/"):
        print("It seems dart is not installed.Ignored")


if __name__ == "__main__":
    main()
