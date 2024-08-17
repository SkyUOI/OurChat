import datetime
import os
import sys
from concurrent.futures import ThreadPoolExecutor, wait
from logging import getLogger
from typing import Any, List

import rmodule
from lib.chattingSystem import ChattingSystem
from lib.connection import Connection
from lib.OurChatAccount import OurChatAccount
from lib.OurChatCache import OurChatCache
from lib.OurChatConfig import OurChatConfig
from lib.OurChatLanguage import OurChatLanguage
from lib.OurChatSession import OurChatSession
from lib.uiSystem import UISystem
from PyQt6.QtWidgets import QMessageBox

logger = getLogger(__name__)


class OurChat:
    def __init__(self) -> None:
        logger.info("OurChat init")
        self.listen_event = {}
        self.tasks = {}
        self.event_queue = []
        self.runQueue = []
        self.version_details = {}
        self.uisystem = None
        self.account = None
        self.accounts_cache = {}
        self.sessions_cache = {}

        self.config = OurChatConfig()
        self.language = OurChatLanguage()
        self.cache = OurChatCache(self)
        self.chatting_system = ChattingSystem(self)
        self.conn = Connection(self)
        self.configUpdated()
        self.cache.connectToDB()
        self.chatting_system.connectToDB()
        self.thread_pool = ThreadPoolExecutor(2)
        self.getVersion()

    def run(self) -> None:
        logger.info("OurChat UI Run")
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.configUpdated()
        self.uisystem.run()
        self.uisystem.exec()

    def runThread(self, task, func: Any | None = None, *args) -> None:
        logger.info(f"OurChat RunThread {task.__name__}")
        logger.debug(f"OurChat RunThread {task.__name__} args:{args}")
        future = self.thread_pool.submit(task, *args)
        self.tasks[future] = func

    def tick(self) -> None:
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

        # event
        for i in range(len(self.event_queue)):
            data = self.event_queue[-1]
            logger.info("deal with event")
            logger.debug(f"deal with event(data:{data})")
            self.event_queue.pop(-1)
            if data["code"] not in self.listen_event:
                continue
            for func in self.listen_event[data["code"]]:
                logger.info(f"run {func.__name__}")
                func(data)

        # later
        for func in self.runQueue:
            logger.info(f"run {func.__name__}")
            func()
        self.runQueue.clear()

    def close(self) -> None:
        logger.info("OurChat begin to close")
        self.uisystem.close()
        self.conn.close()
        self.chatting_system.close()
        self.cache.close()
        logger.debug("wait for threads")
        wait(self.tasks)
        self.thread_pool.shutdown()
        self.listen_event = {}
        self.tasks = {}
        self.event_queue = []
        self.version_details = {}
        self.chatting_system.close()
        self.config.write()
        logger.info("OurChat has been closed")

    def listen(self, event_code: int, func: Any) -> None:
        logger.info(f"listen to CODE{event_code} for {func.__name__}")
        if event_code not in self.listen_event:
            self.listen_event[event_code] = []
        self.listen_event[event_code].append(func)

    def unListen(self, event_code: int, func: Any) -> None:
        logger.info(f"unlisten to CODE{event_code} for {func.__name__}")
        self.listen_event[event_code].remove(func)

    def triggerEvent(self, data: dict) -> None:
        logger.info("add event to event_queue")
        logger.debug(f"add event to event_queue: {data}")
        self.event_queue.append(data)

    def restart(self, message: str | None = None) -> None:
        logger.info("OurChat restart")
        if message is not None:
            QMessageBox.information(
                self.uisystem.mainwindow,
                self.language["restart"],
                self.language["restart_reason"].format(message),
            )
        self.close()

        # start again
        self.thread_pool = ThreadPoolExecutor(2)
        self.configUpdated()
        self.cache.connectToDB()
        self.chatting_system.connectToDB()
        self.getVersion()
        self.uisystem.run()

    def clearLog(self) -> None:
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

    def getLanguages(self) -> List[str]:
        language_files = os.listdir("lang")
        return [language_file.replace(".lang", "") for language_file in language_files]

    def configUpdated(self) -> None:
        self.language.setPath("lang", f'{self.config["general"]["language"]}.lang')
        self.language.read()
        self.conn.setServer(self.config["server"]["ip"], self.config["server"]["port"])
        if self.uisystem is not None:
            self.uisystem.configUpdated()

    def getVersion(self) -> None:
        version_details = rmodule.version_details.split("\n")
        self.version_details = {}
        for i in version_details:
            if ":" not in i:
                continue
            index = i.index(":")
            key = i[:index]
            value = i[index + 1 :]
            self.version_details[key] = value

    def runInMainThread(self, func: Any) -> None:
        self.runQueue.append(func)

    def setAccount(self, account: OurChatAccount) -> None:
        self.account = account

    def getAccount(self, ocid: str, me: bool = False) -> OurChatAccount:
        if ocid not in self.accounts_cache:
            account = OurChatAccount(self, ocid, me)
            self.accounts_cache[ocid] = account
        return self.accounts_cache[ocid]

    def getSession(self, session_id: str) -> OurChatSession:
        if session_id not in self.sessions_cache:
            session = OurChatSession(self, session_id)
            self.sessions_cache[session_id] = session
        return self.sessions_cache[session_id]
