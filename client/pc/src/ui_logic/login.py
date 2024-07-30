from ui.login import Ui_Login as Ui_Login_NOLOGIC
import asyncio


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
        print(asyncio.run(self.ourchat.connectToServer()))
