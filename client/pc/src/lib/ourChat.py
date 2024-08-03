from lib.connenction import Connection
from lib.uiSystem import UISystem
from ui_logic.main import Ui_Main
from ui_logic.login import Ui_Login
from concurrent.futures import ThreadPoolExecutor, wait
from logging import getLogger, INFO
from PyQt6.QtWidgets import QMessageBox
import sys
import os
import json
import datetime

logger = getLogger(__name__)


class OurChat:
    def __init__(self):
        logger.info("OurChat init")
        self.config = OurChatConfig()
        self.conn = Connection(self)
        self.conn.setServer(self.config["server"]["ip"], self.config["server"]["port"])
        self.uisystem = None
        self.thread_pool = ThreadPoolExecutor(2)
        self.listen_message = {}
        self.tasks = {}
        self.message_queue = []

    def run(self):
        logger.info("OurChat UI Run")
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.setUI(Ui_Main)
        widget = self.uisystem.setWidget(Ui_Login, True)
        widget.show()
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

    def close(self):
        logger.info("OurChat begin to close")
        logger.debug("close connection")
        self.conn.close()
        logger.debug("wait for threads")
        wait(self.tasks)
        self.thread_pool.shutdown()
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
        if message is not None:
            QMessageBox.information(f"Because {message}.\nOutChat will restart later")
        self.uisystem.app.closeAllWindows()
        self.uisystem.setUI(Ui_Main)
        dialog = self.uisystem.setWidget(Ui_Login, True)
        dialog.show()

    def clearLog(self):
        logger.info("start to clear log")
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


class OurChatAccount:
    pass


class OurChatConfig:
    def __init__(self, path="./", filename="config.json"):
        self.path = path
        self.filename = filename
        self.read()

    def defaultConfig(self):
        return {
            "server": {"ip": "127.0.0.1", "port": 7777},
            "general": {"theme": "dark_amber", "language": "en_us"},
            "advanced": {"log_level": INFO, "log_saving_limit": 30},
        }

    def write(self, data):
        logger.info(f"write config to {os.path.join(self.path,self.filename)}")
        with open(os.path.join(self.path, self.filename), "w") as f:
            json.dump(data, f, indent=1)

    def read(self):
        logger.info(f"read config from {os.path.join(self.path,self.filename)}")
        if not os.path.exists(os.path.join(self.path, self.filename)):
            logger.info(
                f"{os.path.join(self.path,self.filename)} not exist, use default config"
            )
            self.config = self.defaultConfig()
            self.write(self.config)
        with open(os.path.join(self.path, self.filename), "r") as f:
            self.config = json.load(f)

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
        self.write(self.config)
