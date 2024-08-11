from lib.OurChat import OurChat
import datetime
from logging import getLogger, basicConfig
from lib.OurChat import OurChatConfig
import os

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
