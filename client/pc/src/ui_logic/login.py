from ui.login import Ui_Login as Ui_Login_NOLOGIC
import asyncio
from PyQt5.QtCore import QThread

class AsyncConnectionSpawner(QThread):
    def __init__(self, parent=None):
        super().__init__(parent)

    def init(self, cor):
        self.cor = cor
    
    def run(self):
        asyncio.run(self.cor)

class Ui_Login(Ui_Login_NOLOGIC):
    def __init__(self, ourchat, dialog):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.dialog = dialog

    def setupUi(self):
        super().setupUi(self.dialog)
        self.join_btn.setEnabled(False)
        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        self.join_btn.clicked.connect(self.join)
        self.connect_server_btn.clicked.connect(self.connectToServer)

    def join(self):
        index = self.tabWidget.currentIndex()
        if index:  # register
            pass
        else:  # login
            pass

    def connectToServer(self):
        self.ourchat.setServer(
            self.server_ip_editor.text(), self.server_port_editor.text()
        )
        async_thread = AsyncConnectionSpawner(self.connect_server_btn)
        async_thread.init(self.ourchat.connectToServer())
        async_thread.start()
