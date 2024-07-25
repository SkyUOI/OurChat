"""
用于初始化rust的valgrind配置，主要用于CI/CD的valgrind检测单元测试
"""

import os

dir_name = os.path.expanduser("~/.cargo/config.toml")
with open(dir_name, "w") as f:
    _ = f.write(
        """
[target.'cfg(target_os = "linux")']
runner = "valgrind --leak-check=full"
    """
    )
