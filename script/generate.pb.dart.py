import os

commands = []


def generate(dir, from_path, to_path):
    path = os.getcwd()
    os.chdir(dir)
    for file in os.listdir():
        if os.path.isdir(file):
            generate(file, from_path + "/" + file, to_path)
        else:
            commands.append(f"protoc {from_path}/{file} --dart_out=grpc:{to_path}")

    os.chdir(path)


generate("service", "service", "client/ourchat/lib")
for index in range(len(commands)):
    command = commands[index]
    os.system(command)
    print(
        f"\r|{'#'*int(index/len(commands)*100/5)}{' '*(20-(int(index/len(commands)*100/5)))}| {round(index/len(commands)*100,2)}%",
        end="",
    )
os.system(
    "protoc --dart_out=grpc:client/ourchat/lib google/protobuf/timestamp.proto google/protobuf/empty.proto"
)
os.system("dart format client/ourchat/lib/google")
os.system("dart format client/ourchat/lib/service")
