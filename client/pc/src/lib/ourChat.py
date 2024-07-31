from lib.connenction import Connection
from lib.uiSystem import UISystem
from ui_logic.main import Ui_Main
from ui_logic.login import Ui_Login
from concurrent.futures import ThreadPoolExecutor, wait
from logging import getLogger
from PyQt5.QtWidgets import QMessageBox
import hashlib
import sys

logger = getLogger(__name__)


class OurChat:
    def __init__(self):
        logger.info("OurChat init")
        self.conn = Connection(self)
        self.uisystem = None
        self.thread_pool = ThreadPoolExecutor(2)
        self.listen_message = {}
        self.tasks = {}
        self.message_queue = []

    def run(self):
        logger.info("OurChat UI Run")
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.setUI(Ui_Main)
        dialog = self.uisystem.setDialog(Ui_Login, True)
        dialog = self.uisystem.setDialog(Ui_Login, True)
        dialog.show()
        self.uisystem.exec()

    def runThread(self, task, func=None, *args):
        logger.info(f"OurChat RunThread {task.__name__}")
        logger.debug(f"OurChat RunThread {task.__name__} args:{args}")
        future = self.thread_pool.submit(task, *args)
        self.tasks[future] = func

    def tick(self):
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
        dialog = self.uisystem.setDialog(Ui_Login, True)
        dialog.show()


class OurChatAccount:
    def __init__(self, ourchat: OurChat):
        self.ourchat = ourchat
        self.ocid = None

    def register(self, email, password):
        sha256 = hashlib.sha256()
        sha256.update(password.encode("ascii"))
        encoded_password = sha256.hexdigest()
        data = {"code": 4, "email": email, "password": encoded_password}
        self.ourchat.sendData(data)
