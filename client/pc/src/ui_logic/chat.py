from ui.chat import Ui_Chat as Ui_Chat_NOLOGIC
from lib.OurChatUI import SessionList


class Ui_Chat(Ui_Chat_NOLOGIC):
    def __init__(self, ourchat, widget):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(
        self,
    ):
        super().setupUi(self.widget)
        self.session_list.deleteLater()
        self.session_list = SessionList(self.widget)
        self.session_list.addSession("resources/images/test.jpg", "Name", "detail")
        self.left_panel.addWidget(self.session_list)
        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        pass
