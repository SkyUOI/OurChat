from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtWidgets import QMainWindow, QApplication, QDialog, QMessageBox
from PyQt5.QtGui import QPixmap
from PyQt5.QtCore import QTimer
from ui import main_window, login
import sys, asyncio


class MainUi(main_window.Ui_MainWindow):
    def init(self):
        pass

    def rename(self):
        pass

    def bind(self):
        pass

    def setupUi(self, ui_system):
        self.uisystem = ui_system
        super().setupUi(ui_system)


class Login(login.Ui_MainWindow):
    def init(self):
        self.rename()
        self.bind()

    def rename(self):
        img = QPixmap("./resource/startImg.png")
        self.img.setPixmap(img)

    def bind(self):
        self.login.clicked.connect(self.tryLogin)
        self.show_paw_checkbox.stateChanged.connect(self.showPassowrd)

    def setupUi(self, ui_system):
        self.uisystem = ui_system
        super().setupUi(ui_system)

    def tryLogin(self):
        account = self.account.text()
        password = self.password.text()
        self.uisystem.mainsystem.login(account, password)

    def showPassowrd(self):
        is_check = self.show_paw_checkbox.isChecked()
        if is_check:
            self.password.setEchoMode(QtWidgets.QLineEdit.Normal)
        else:
            self.password.setEchoMode(QtWidgets.QLineEdit.Password)


class UiSystem(QMainWindow):
    def __init__(self, mainsystem):
        super().__init__()
        self.ui = None
        self.dialog = None
        self.dialog_ui = None
        self.mainsystem = mainsystem
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.tick)
        self.timer.start(100)

    def showUi(self, ui):
        self.ui = ui
        self.ui.setupUi(self)
        self.ui.init()
        self.show()

    def showDialog(self, dialog_ui):
        self.dialog_ui = dialog_ui
        if self.dialog is not None:
            self.dialog.close()
            self.dialog = None
        self.dialog = QDialog(self)
        self.dialog_ui.setupUi(self.dialog)
        self.dialog_ui.init(self)
        self.dialog.show()

    def closeDialog(self):
        if self.dialog is not None:
            self.dialog.close()
            self.dialog = None
            self.dialog_ui = None

    def tick(self):
        for task in self.mainsystem.task_queue:
            task()
            self.mainsystem.task_queue.pop(0)
