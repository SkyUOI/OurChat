import datetime
import hashlib
import os
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor, wait
from logging import getLogger
from typing import Any, List
from urllib.error import HTTPError

import rmodule
from lib.chattingSystem import ChattingSystem
from lib.connection import Connection
from lib.const import (
    DOWNLOAD_RESPONSE,
    HTTP_OK,
    REQUEST_INFO_NOT_FOUND,
    RUN_NORMALLY,
    UPLOAD_MSG,
    UPLOAD_RESPONSE,
    UPLOAD_RESPONSE_MSG,
)
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
        self.tasks = []
        self.event_queue = []
        self.runQueue = []
        self.version_details = {}
        self.uisystem = None
        self.account = None
        self.accounts_cache = {}
        self.sessions_cache = {}
        self.upload_queue = {}

        self.config = OurChatConfig()
        self.language = OurChatLanguage()
        self.cache = OurChatCache(self)
        self.chatting_system = ChattingSystem(self)
        self.conn = Connection(self)
        self.configUpdated()
        self.cache.connectToDB()
        self.thread_pool = ThreadPoolExecutor(4)
        self.getVersion()

    def run(self) -> None:
        logger.info("OurChat UI Run")
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.configUpdated()
        self.uisystem.run()
        self.uisystem.exec()

    def runThread(self, task, *args) -> None:
        logger.info(f"OurChat RunThread {task.__name__}")
        logger.debug(f"OurChat RunThread {task.__name__} args:{args}")
        future = self.thread_pool.submit(task, *args)
        future.task_name = task.__name__
        self.tasks.append(future)

    def tick(self) -> None:
        # threads
        remove_ = []
        for future in self.tasks:
            if future.done():
                logger.info(f"A task had done. ({future.task_name})")
                remove_.append(future)
        for r in remove_:
            self.tasks.remove(r)

        # event
        for i in range(len(self.event_queue)):
            data = self.event_queue[-1]
            logger.info("deal with event")
            logger.debug(f"deal with event (data:{data})")
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
        self.tasks = []
        self.event_queue = []
        self.version_details = {}
        self.accounts_cache = {}
        self.sessions_cache = {}
        self.upload_queue = {}
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
        self.thread_pool = ThreadPoolExecutor(4)
        self.configUpdated()
        self.cache.connectToDB()
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
        self.chatting_system.connectToDB(f"record_{account.ocid}.db")

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

    def download(self, url: str, key: str, depth: int = 0) -> None:
        if depth == 0:
            logger.info(f"begin to download (URL: {url})")
        if depth >= 5:
            logger.warning(f"download failed (URL: {url})")
            self.triggerEvent(
                {
                    "code": DOWNLOAD_RESPONSE,
                    "status": REQUEST_INFO_NOT_FOUND,
                    "url": url,
                }
            )
            return
        request = urllib.request.Request(url, headers={"Key": key}, method="POST")
        try:
            logger.info(f"download (URL: {url})(retry: {depth})")
            data = urllib.request.urlopen(request).read()
            logger.info(f"download success (URL: {url})")
            self.triggerEvent(
                {
                    "code": DOWNLOAD_RESPONSE,
                    "status": RUN_NORMALLY,
                    "data": data,
                    "url": url,
                }
            )
        except Exception as e:
            logger.warning(f"download failed({str(e)})")
            logger.info(f"retry after 3s (URL: {url})")
            time.sleep(3)
            self.download(url, key, depth + 1)

    def upload(self, data: bytes, auto_clean=False) -> None:
        sha256 = hashlib.sha256()
        sha256.update(data)
        logger.info(f"upload file (hash: {sha256.hexdigest()})")
        self.upload_queue[sha256.hexdigest()] = data
        self.listen(UPLOAD_RESPONSE_MSG, self.uploadResponse)
        self.conn.send(
            {"code": UPLOAD_MSG, "hash": sha256.hexdigest(), "auto_clean": auto_clean}
        )

    def uploadResponse(self, data: dict) -> None:
        self.unListen(UPLOAD_RESPONSE_MSG, self.uploadResponse)
        if data["status"] == RUN_NORMALLY:
            file_data = self.upload_queue[data["hash"]]
            self.upload_queue.pop(data["hash"])
            key = data["key"]
            request = urllib.request.Request(
                url=f"http://{self.config['server']['ip']}:{self.config['server']['port']+1}/upload",
                data=file_data,
                headers={"Key": key},
                method="POST",
            )
            try:
                response = urllib.request.urlopen(request)
                if response.code == HTTP_OK:
                    logger.info(f"get upload response: success (hash: {data['hash']})")
                    self.triggerEvent(
                        {
                            "code": UPLOAD_RESPONSE,
                            "status": HTTP_OK,
                            "hash": data["hash"],
                            "key": key,
                        }
                    )
            except HTTPError as he:
                code = he.code
                logger.warning(
                    f"get upload response code: {code} (hash: {data['hash']})"
                )
                self.triggerEvent(
                    {"code": UPLOAD_RESPONSE, "status": code, "hash": data["hash"]}
                )
