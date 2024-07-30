from ui.setting import Ui_Setting as Ui_Setting_NOLOGIC


class Ui_Setting(Ui_Setting_NOLOGIC):
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
