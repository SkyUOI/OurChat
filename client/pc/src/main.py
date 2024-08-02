from lib.OurChat import OurChat
import datetime
from logging import getLogger, basicConfig, INFO
import os

if "log" not in os.listdir():
    os.mkdir("log")

logger = getLogger(__name__)
basicConfig(
    filename=f'log/{datetime.datetime.strftime(datetime.datetime.now(),"%Y-%m-%d")}.log',
    level=INFO,
    encoding="utf-8",
)

ourchat = OurChat()
ourchat.run()
ourchat.close()
