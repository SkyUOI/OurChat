from ui.account import Ui_Account as Ui_Account_NOLOGIC


class Ui_Account(Ui_Account_NOLOGIC):
    def __init__(self, ourchat, widget):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(self):
        super().setupUi(self.widget)
        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        pass
