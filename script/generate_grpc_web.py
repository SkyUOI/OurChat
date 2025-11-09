import os

commands = []


def generate(dir):
    for root, dirs, files in os.walk(dir):
        for file in files:
            commands.append(
                f"pnpx protoc {os.path.join(root, file)} \
--ts_out=server/web-panel/src/api \
--ts_opt eslint_disable"
            )


generate("service")
for index in range(len(commands)):
    command = commands[index]
    os.system(command)
    block = int(index / len(commands) * 100 / 5)
    print(
        f"\r|{'#' * block}{' ' * (20 - block)}| {round(index / len(commands) * 100, 2)}%",
        end="",
    )
print(f"\r|{'#' * 20}| 100.00%")
