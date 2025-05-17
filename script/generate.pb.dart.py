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


generate("service", "service", "client/lib")
for index in range(len(commands)):
    command = commands[index]
    os.system(command)
    block = int(index / len(commands) * 100 / 5)
    print(
        f"\r|{'#' * block}{' ' * (20 - block)}| {round(index / len(commands) * 100, 2)}%",
        end="",
    )
print(f"\r|{'#' * 20}| 100.00%")
os.system(
    "protoc --dart_out=grpc:client/lib \
    google/protobuf/timestamp.proto \
    google/protobuf/empty.proto \
    google/protobuf/duration.proto"
)
os.system("dart format client/lib/google")
os.system("dart format client/lib/service")
