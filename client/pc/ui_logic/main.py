from ui.main import Ui_Main as Ui_Main_NOLOGIC
from PyQt5.QtWidgets import QWidget
from ui_logic.chat import Ui_Chat
from ui_logic.setting import Ui_Setting
from ui_logic.account import Ui_Account


class Ui_Main(Ui_Main_NOLOGIC):
    def __init__(self, uisystem) -> None:
        self.uisystem = uisystem
        self.mainwindow = self.uisystem.mainwindow
        self.widget = None

    def setupUi(self):
        super().setupUi(self.mainwindow)

        # self.widget = QWidget(self.mainwindow)

        self.setWidget(Ui_Chat)

        self.fillText()
        self.bind()

    def setWidget(self, ui):
        if self.widget is not None:
            self.verticalLayout_2.removeWidget(self.widget)
        self.widget = QWidget(self.mainwindow)
        widget_ui = ui(self.uisystem, self.widget)
        widget_ui.setupUi()
        self.verticalLayout_2.addWidget(self.widget)
        self.widget.show()

    def bind(self):
        self.to_chat.clicked.connect(lambda: self.setWidget(Ui_Chat))
        self.to_account.clicked.connect(lambda: self.setWidget(Ui_Account))
        self.to_setting.clicked.connect(lambda: self.setWidget(Ui_Setting))

    def fillText(self):
        self.mainwindow.setWindowTitle("OurChat")
