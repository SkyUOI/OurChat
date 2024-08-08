from ui.session import Ui_Session as Ui_Session_NOLOGIC
from lib.OurChatUI import SessionWidget, MessageWidget
from PyQt6.QtWidgets import QListWidgetItem
from PyQt6.QtCore import QSize


class Ui_Session(Ui_Session_NOLOGIC):
    def __init__(self, ourchat, widget):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(
        self,
    ):
        super().setupUi(self.widget)
        self.message_list.verticalScrollBar().setSingleStep(10)
        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        pass

    def addSession(self, avatar_path, name, detail):
        item = QListWidgetItem(self.session_list)
        item.setSizeHint(QSize(65, 65))
        widget = SessionWidget(self.session_list)
        widget.setSession(avatar_path, name, detail)
        self.session_list.addItem(item)
        self.session_list.setItemWidget(item, widget)

    def addMessage(self, avatar_path, name, message, me):
        item = QListWidgetItem(self.message_list)
        widget = MessageWidget(self.message_list)
        widget.setMessage(item, avatar_path, name, message, me)
        self.message_list.addItem(item)
        self.message_list.setItemWidget(item, widget)
