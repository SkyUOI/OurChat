from lib.connenction import Connection
from lib.uiSystem import UISystem
from concurrent.futures import ThreadPoolExecutor, wait
from logging import getLogger, DEBUG, INFO, WARNING, CRITICAL, ERROR
from PyQt6.QtWidgets import QMessageBox
from copy import deepcopy
from lib.chattingSystem import ChattingSystem
import sys
import os
import json
import datetime
import re
import rmodule

logger = getLogger(__name__)


class OurChat:
    def __init__(self):
        logger.info("OurChat init")
        self.listen_message = {}
        self.tasks = {}
        self.message_queue = []
        self.runQueue = []
        self.version_details = {}
        self.uisystem = None
        self.chatting_system = ChattingSystem(self)

        self.config = OurChatConfig()
        self.language = OurChatLanguage()
        self.conn = Connection(self)
        self.configUpdated()
        self.thread_pool = ThreadPoolExecutor(2)
        self.getVersion()

    def run(self):
        logger.info("OurChat UI Run")
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.configUpdated()
        self.uisystem.run()
        self.uisystem.exec()

    def runThread(self, task, func=None, *args):
        logger.info(f"OurChat RunThread {task.__name__}")
        logger.debug(f"OurChat RunThread {task.__name__} args:{args}")
        future = self.thread_pool.submit(task, *args)
        self.tasks[future] = func

    def tick(self):
        # threads
        remove_ = []
        tasks = list(self.tasks.keys())
        for future in tasks:
            if future.done():
                logger.info(f"A task had done. result: {future.result()}")
                func = self.tasks[future]
                if func is not None:
                    logger.info(f"call function: {func.__name__}({future.result()})")
                    func(future.result())
                remove_.append(future)
        for r in remove_:
            self.tasks.pop(r)

        # message
        for i in range(len(self.message_queue)):
            data = self.message_queue[-1]
            logger.info("deal with message data")
            logger.debug(f"deal with message data: {data}")
            self.message_queue.pop(-1)
            for func in self.listen_message[data["code"]]:
                logger.info(f"run {func.__name__}")
                func(data)

        # later
        for func in self.runQueue:
            logger.info(f"run {func.__name__}")
            func()
        self.runQueue.clear()

    def close(self):
        logger.info("OurChat begin to close")
        self.uisystem.app.closeAllWindows()
        self.conn.close()
        logger.debug("wait for threads")
        wait(self.tasks)
        self.thread_pool.shutdown()
        self.listen_message = {}
        self.tasks = {}
        self.message_queue = []
        self.version_details = {}
        self.chatting_system.close()
        self.config.write()
        logger.info("OurChat has been closed")

    def listen(self, message_code, func):
        logger.info(f"listen to CODE{message_code} for {func.__name__}")
        if message_code not in self.listen_message:
            self.listen_message[message_code] = []
        self.listen_message[message_code].append(func)

    def unListen(self, message_code, func):
        logger.info(f"unlisten to CODE{message_code} for {func.__name__}")
        self.listen_message[message_code].remove(func)

    def getMessage(self, data):
        logger.info("add message to message_queue")
        logger.debug(f"add message to message_queue: {data}")
        self.message_queue.append(data)

    def restart(self, message=None):
        logger.info("OurChat restart")
        if message is not None:
            QMessageBox.information(
                self.uisystem.mainwindow,
                self.language["restart"],
                self.language["restart_reason"].format(message),
            )
        self.close()
        self.thread_pool = ThreadPoolExecutor(2)
        self.configUpdated()
        self.getVersion()
        self.uisystem.run()

    def clearLog(self):
        logger.info("start to clear log")
        if self.config["advanced"]["log_saving_limit"] == -1:
            return
        logs = os.listdir("log")
        logs.sort()
        for log in logs:
            date = datetime.datetime.strptime(
                log.replace(".log", ""), "%Y-%m-%d"
            ).date()
            now = datetime.datetime.now().date()
            days = (now - date).days
            if days > self.config["advanced"]["log_saving_limit"]:
                logger.info(f"remove log {log}")
                os.remove(os.path.join("log", log))

    def getLanguages(self):
        language_files = os.listdir("lang")
        return [language_file.replace(".lang", "") for language_file in language_files]

    def configUpdated(self):
        self.language.setPath("lang", f'{self.config["general"]["language"]}.lang')
        self.language.read()
        self.conn.close()
        self.conn.setServer(self.config["server"]["ip"], self.config["server"]["port"])
        self.chatting_system.connectToDB()
        if self.uisystem is not None:
            self.uisystem.configUpdated()

    def getVersion(self):
        version_details = rmodule.version_details.split("\n")
        self.version_details = {}
        for i in version_details:
            if ":" not in i:
                continue
            index = i.index(":")
            key = i[:index]
            value = i[index + 1 :]
            self.version_details[key] = value

    def runInMainThread(self, func):
        self.runQueue.append(func)


class OurChatAccount:
    pass


class OurChatConfig:
    def __init__(self, path="./", filename="config.json"):
        self.path = path
        self.filename = filename
        self.read()

    def defaultConfig(self):
        return {
            "server": {"ip": "127.0.0.1", "port": 7777, "reconnection_attempt": 5},
            "general": {"theme": "dark_amber", "language": "en-us"},
            "advanced": {"log_level": INFO, "log_saving_limit": 30},
        }

    def write(self):
        logger.info(f"write config to {os.path.join(self.path,self.filename)}")
        self.check()
        with open(os.path.join(self.path, self.filename), "w") as f:
            json.dump(self.config, f, indent=1)

    def read(self):
        logger.info(f"read config from {os.path.join(self.path,self.filename)}")
        if not os.path.exists(os.path.join(self.path, self.filename)):
            logger.info(
                f"{os.path.join(self.path,self.filename)} not exist, use default config"
            )
            self.config = self.defaultConfig()
            self.write()
        with open(os.path.join(self.path, self.filename), "r") as f:
            self.config = json.load(f)
        self.check()

    def __getitem__(self, key):
        if key not in self.config.keys():
            default = self.defaultConfig()
            if key in default.keys():
                self[key] = default[key]
                return self[key]
            else:
                return None
        return self.config[key]

    def __setitem__(self, key, value):
        self.config[key] = value

    def checkType(self, value_type=None, default_value=None, config=None):
        if value_type is None:
            value_type = {
                "server": {"ip": str, "port": int, "reconnection_attempt": int},
                "general": {"theme": str, "language": str},
                "advanced": {"log_level": int, "log_saving_limit": int},
            }
            default_value = self.defaultConfig()
            config = self.config

        if isinstance(default_value, dict):
            for key in value_type.keys():
                if key not in config.keys():
                    config[key] = default_value[key]
                else:
                    config[key] = self.checkType(
                        value_type[key], default_value[key], config[key]
                    )
            return config
        else:
            if isinstance(config, value_type):
                return config
            else:
                return default_value

    def check(self):
        logger.info("check config")
        self.checkType()

        default = self.defaultConfig()
        ip = re.match(
            "^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
            self["server"]["ip"],
        )
        if ip is None:
            self["server"]["ip"] = default["server"]["ip"]
        if self["server"]["port"] < 1 or self["server"]["port"] > 65535:
            self["server"]["port"] = default["server"]["port"]

        themes = os.listdir("theme")
        if self["general"]["theme"] not in themes:
            if default["general"]["theme"] in themes:
                self["general"]["theme"] = default["general"]["theme"]
            elif len(themes) > 0:
                self["general"]["theme"] = themes[0]
            else:
                self["general"]["theme"] = None

        languages = [language.replace(".lang", "") for language in os.listdir("lang")]
        if self["general"]["language"] not in languages:
            if default["general"]["language"] in languages:
                self["general"]["language"] = default["general"]["language"]
            elif len(languages) > 0:
                self["general"]["language"] = languages[0]
            else:
                self["general"]["language"] = None

        if self["advanced"]["log_level"] not in [DEBUG, INFO, WARNING, ERROR, CRITICAL]:
            self["advanced"]["log_level"] = default["advanced"]["log_level"]

        if (
            self["advanced"]["log_saving_limit"] < 1
            and self["advanced"]["log_saving_limit"] != -1
        ):
            self["advanced"]["log_saving_limit"] = default["advanced"][
                "log_saving_limit"
            ]

    def setConfig(self, config):
        if isinstance(config, dict):
            self.config = deepcopy(config)
        elif isinstance(config, OurChatConfig):
            self.config = deepcopy(config.config)

    def compareConfig(self, config):
        if isinstance(config, dict):
            return self.config == config
        elif isinstance(config, OurChatConfig):
            return self.config == config.config


class OurChatLanguage:
    def __init__(self, path="./lang", filename="en-us.lang"):
        self.setPath(path, filename)
        self.translate = {}

    def setPath(self, path, filename):
        self.path = path
        self.filename = filename

    def read(self):
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

    def __getitem__(self, key):
        if key not in self.translate.keys():
            return key
        return self.translate[key]
