from lib.connenction import Connection
from lib.uiSystem import UISystem
from ui_logic.main import Ui_Main
from ui_logic.login import Ui_Login
from concurrent.futures import ThreadPoolExecutor, wait
import hashlib
import sys


class OurChat:
    def __init__(self):
        self.conn = Connection()
        self.uisystem = None
        self.thread_pool = ThreadPoolExecutor(2)
        self.tasks = {}

    def run(self):
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.setUI(Ui_Main)
        self.uisystem.mainwindow.setEnabled(False)
        dialog = self.uisystem.setDialog(Ui_Login, True)
        dialog.show()
        self.uisystem.exec()

    def runThread(self, task, func=None, *args):
        future = self.thread_pool.submit(task, *args)
        self.tasks[future] = func

    def tick(self):
        remove_ = []
        tasks = list(self.tasks.keys())
        for future in tasks:
            if future.done():
                func = self.tasks[future]
                if func is not None:
                    func(future.result())
                remove_.append(future)
        for r in remove_:
            self.tasks.pop(r)

    def close(self):
        self.conn.close()
        wait(list(self.tasks.keys()))
        self.thread_pool.shutdown()


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
