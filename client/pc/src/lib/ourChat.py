from lib.connenction import Connection
from lib.uiSystem import UISystem
from ui_logic.main import Ui_Main
from ui_logic.login import Ui_Login
from lib.msg_code import GENERATE_VERIFY
import hashlib
import sys


class OurChat:
    def __init__(self):
        self.conn = None
        self.uisystem = None

    def setServer(self, ip, port):
        self.ip = ip
        self.port = port

    async def connectToServer(self):
        self.conn = Connection(self.ip, self.port)
        response = await self.conn.connect()
        return response

    async def sendData(self, data):
        return await self.conn.send(data)

    def gentateVerify(self):
        self.sendData({"code": GENERATE_VERIFY})

    def run(self):
        self.uisystem = UISystem(self, sys.argv)
        self.uisystem.setUI(Ui_Main)
        self.uisystem.mainwindow.setEnabled(False)
        dialog = self.uisystem.setDialog(Ui_Login, True)
        dialog.show()
        self.uisystem.exec()


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
