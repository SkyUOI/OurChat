from ui.main import Ui_Main as Ui_Main_NOLOGIC


class Ui_Main(Ui_Main_NOLOGIC):
    def __init__(self, uisystem) -> None:
        self.uisystem = uisystem
        self.mainwindow = self.uisystem.mainwindow

    def setupUi(self):
        super().setupUi(self.mainwindow)
        desktop_size = self.uisystem.app.desktop().size()
        self.mainwindow.resize(
            int(desktop_size.height()), int(desktop_size.height() / 1.5)
        )
        self.fillText()
        self.bind()

    def bind(self):
        pass

    def fillText(self):
        self.mainwindow.setWindowTitle("OurChat")
