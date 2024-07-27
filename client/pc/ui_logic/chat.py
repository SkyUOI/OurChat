from ui.chat import Ui_Chat as Ui_Chat_NOLOGIC


class Ui_Chat(Ui_Chat_NOLOGIC):
    def __init__(self, uisystem, widget):
        self.uisystem = uisystem
        self.widget = widget

    def setupUi(
        self,
    ):
        super().setupUi(self.widget)

        self.fillText()
        self.bind()
    def fillText(self):
        pass

    def bind(self):
        pass
