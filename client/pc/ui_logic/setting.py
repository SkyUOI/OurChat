from ui.setting import Ui_Setting as Ui_Setting_NOLOGIC

class Ui_Setting(Ui_Setting_NOLOGIC):
    def __init__(self, uisystem, widget):
        self.uisystem = uisystem
        self.widget = widget
    def setupUi(self):
        super().setupUi(self.widget)
        self.fillText()
        self.bind()
    def fillText(self):
        pass

    def bind(self):
        pass
