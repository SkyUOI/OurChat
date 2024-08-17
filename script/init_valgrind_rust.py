"""
用于初始化rust的valgrind配置,主要用于CI/CD的valgrind检测单元测试
"""

import os


def init_valgrind():
    path_base = os.path.expanduser("~/.cargo/")
    if not os.path.exists(path_base):
        os.makedirs(path_base)
    toml_name = os.path.expanduser("~/.cargo/config.toml")
    with open(toml_name, "w") as f:
        _ = f.write(
            """
[target.'cfg(target_os = "linux")']
runner = "valgrind --leak-check=full"
    """
        )


if __name__ == "__main__":
    init_valgrind()
