from ui.login import Ui_Login as Ui_Login_NOLOGIC

class Ui_Login(Ui_Login_NOLOGIC):
    def __init__(self,uisystem,dialog):
        self.uisystem = uisystem
        self.dialog = dialog
    
    def setupUi(self):
        super().setupUi(self.dialog)
        self.fillText()
        self.bind()
    
    def fillText(self):
        pass

    def bind(self):
        pass