import datetime
import os
from logging import basicConfig, getLogger

from lib.OurChat import OurChat
from lib.OurChatConfig import OurChatConfig

if "log" not in os.listdir():
    os.mkdir("log")

config = OurChatConfig()
config.read()
logger = getLogger(__name__)
basicConfig(
    filename=f'log/{datetime.datetime.strftime(datetime.datetime.now(),"%Y-%m-%d")}.log',
    level=config["advanced"]["log_level"],
    encoding="utf-8",
    format="%(asctime)s %(levelname)s:%(name)s:%(message)s",
    datefmt="%H:%M:%S",
)
logger.info("-" * 30 + "Start to log" + "-" * 30)
ourchat = OurChat()
ourchat.run()
ourchat.close()
ourchat.clearLog()
logger.info("-" * 30 + "Over" + "-" * 30)

# 删除runThread()的func参数
# 删除image_hash 如: avatar_hash
# 修复download到达5次后仍然继续尝试download的bug
# 重写头像逻辑
# 修复login_failed显示错误的bug
