import os
from logging import getLogger

logger = getLogger(__name__)


class OurChatLanguage:
    def __init__(self, path: str = "./lang", filename: str = "en-us.lang") -> None:
        self.setPath(path, filename)
        self.translate = {}

    def setPath(self, path: str, filename: str) -> None:
        logger.info("set language path to " + os.path.join(path, filename))
        self.path = path
        self.filename = filename

    def read(self) -> None:
        logger.info("read language file from " + os.path.join(self.path, self.filename))
        self.translate = {}
        with open(os.path.join(self.path, self.filename), "r", encoding="utf-8") as f:
            for line in f.readlines():
                line = line.strip()
                line = line.split("#")[0]
                if "=" not in line:
                    continue
                key, value = line.split("=")
                key, value = key.strip(), value.strip()
                self.translate[key] = value

    def __getitem__(self, key: str) -> str:
        if key not in self.translate.keys():
            return key
        return self.translate[key]
