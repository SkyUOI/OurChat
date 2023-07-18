from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtWidgets import QMainWindow, QApplication, QDialog, QMessageBox
from PyQt5.QtGui import QPixmap
from PyQt5.QtCore import QTimer
from ui import main_window, login
import sys,asyncio


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


class Login(login.Ui_Dialog):
    def init(self, ui_system):
        self.uisystem = ui_system
        self.rename()
        self.bind()

    def rename(self):
        img = QPixmap("./resource/startImg.png")
        self.img.setPixmap(img)

    def bind(self):
        pass

    def setupUi(self, dialog):
        self.dialog = dialog
        super().setupUi(dialog)


class UiSystem(QMainWindow):
    def __init__(self):
        self.ui = None
        self.dialog = None
        self.dialog_ui = None
        super().__init__()
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.timerFunc)
        self.timer.start(1000)

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

    async def timerFunc(self):
    	await asyncio.sleep(1)


if __name__ == "__main__":
    app = QApplication(sys.argv)
    ui_system = UiSystem()
    ui_system.showDialog(Login())
    ui_system.showUi(MainUi())
    app.exec()
