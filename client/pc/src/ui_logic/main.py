from ui.main import Ui_Main as Ui_Main_NOLOGIC
from PyQt6.QtWidgets import QWidget
from ui_logic.session import Ui_Session
from ui_logic.setting import Ui_Setting
from ui_logic.account import Ui_Account


class Ui_Main(Ui_Main_NOLOGIC):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.mainwindow = self.uisystem.mainwindow
        self.widget = None

    def setupUi(self):
        super().setupUi(self.mainwindow)
        self.setWidget(Ui_Session)
        self.fillText()
        self.bind()

    def setWidget(self, ui):
        if self.widget is not None:
            self.verticalLayout_2.removeWidget(self.widget)
        self.widget = QWidget(self.mainwindow)
        self.widget_ui = ui(self.ourchat, self.widget)
        self.widget_ui.setupUi()
        self.verticalLayout_2.addWidget(self.widget)
        self.widget.show()

    def bind(self):
        self.to_session.clicked.connect(lambda: self.setWidget(Ui_Session))
        self.to_account.clicked.connect(lambda: self.setWidget(Ui_Account))
        self.to_setting.clicked.connect(lambda: self.setWidget(Ui_Setting))

    def fillText(self):
        self.mainwindow.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")
        self.to_session.setText(self.ourchat.language["session"])
        self.to_account.setText(self.ourchat.language["account"])
        self.to_setting.setText(self.ourchat.language["setting"])
