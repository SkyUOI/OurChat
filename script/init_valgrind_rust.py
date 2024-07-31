"""
用于初始化rust的valgrind配置，主要用于CI/CD的valgrind检测单元测试
"""

import os


def init_valgrind():
    dir_name = os.path.expanduser("~/.cargo/config.toml")
    with open(dir_name, "w") as f:
        _ = f.write(
            """
[target.'cfg(target_os = "linux")']
runner = "valgrind --leak-check=full"
    """
        )


if __name__ == "__main__":
    init_valgrind()
