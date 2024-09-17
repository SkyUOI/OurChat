from typing import Any

from PyQt6.QtGui import QKeyEvent
from PyQt6.QtWidgets import QWidget
from ui.main import Ui_Main
from ui_logic import account, session, setting
from ui_logic.basicUI import BasicUI


class MainUI(BasicUI, Ui_Main):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.mainwindow = self.uisystem.mainwindow
        self.widget = None
        self.widget_ui_logic = None

    def setupUi(self) -> None:
        super().setupUi(self.mainwindow)
        self.fillText()
        self.bind()

    def show(self) -> None:
        self.setWidget(session.SessionUI)

    def setWidget(self, ui: Any) -> None:
        if self.widget is not None:
            self.verticalLayout_2.removeWidget(self.widget)
            self.widget_ui_logic.close()
        self.widget = QWidget(self.mainwindow)
        if ui == setting.SettingUI:
            self.widget_ui_logic = ui(self.ourchat, self.widget, True)
        else:
            self.widget_ui_logic = ui(self.ourchat, self.widget)
        self.widget_ui_logic.setupUi()
        self.verticalLayout_2.addWidget(self.widget)
        self.widget.show()

    def bind(self) -> None:
        self.to_session.clicked.connect(lambda: self.setWidget(session.SessionUI))
        self.to_account.clicked.connect(lambda: self.setWidget(account.AccountUI))
        self.to_setting.clicked.connect(lambda: self.setWidget(setting.SettingUI))

    def fillText(self) -> None:
        self.mainwindow.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")
        self.to_session.setText(self.ourchat.language["session"])
        self.to_account.setText(self.ourchat.language["account"])
        self.to_setting.setText(self.ourchat.language["setting"])
        if self.widget_ui_logic is not None:
            self.widget_ui_logic.fillText()

    def keyPressEvent(self, event: QKeyEvent):
        self.widget_ui_logic.keyPressEvent(event)
