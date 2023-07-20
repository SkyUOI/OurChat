from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtWidgets import QMainWindow, QApplication, QDialog, QMessageBox
from PyQt5.QtGui import QPixmap
from PyQt5.QtCore import QTimer
from ui import main_window, login
import sys, asyncio


class MainUi(main_window.Ui_MainWindow):
    def init(self):
        self.messages = []
        self.rename()
        self.showChatList()
        self.bind()

    def rename(self):
        pass

    def bind(self):
        self.chat_list.currentItemChanged.connect(self.clickChat)

    def setupUi(self, ui_system):
        self.uisystem = ui_system
        super().setupUi(ui_system)

    def showChatList(self):
        self.chat_list.clear()
        self.chat_list.addItems(self.uisystem.mainsystem.record.getTableList())

    def showMessage(self):
        self.uisystem.clearLayout(self.scrollArea_layout)
        for message in self.messages:
            is_me = message["is_me"]
            horizontalLayout = QtWidgets.QHBoxLayout()
            horizontalLayout.setObjectName("horizontalLayout")
            spacerItem2 = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum)
            widget = QtWidgets.QWidget(self.scrollAreaWidgetContents)
            gridLayout = QtWidgets.QGridLayout(widget)
            gridLayout.setObjectName("gridLayout")
            message_label = QtWidgets.QLabel(widget)
            message_label.setText(message["msg"])
            if is_me:
                message_label.setAlignment(QtCore.Qt.AlignRight|QtCore.Qt.AlignTrailing|QtCore.Qt.AlignVCenter)
            else:
                message_label.setAlignment(QtCore.Qt.AlignLeft|QtCore.Qt.AlignTrailing|QtCore.Qt.AlignVCenter)
            message_label.setObjectName("label")
            gridLayout.addWidget(message_label, 0, 0, 1, 1)
            
            img_label = QtWidgets.QLabel(self.scrollAreaWidgetContents)
            img_label.setObjectName("img")
            img = QPixmap(message["img"])
            img_label.setPixmap(img)
            if is_me:
                horizontalLayout.addItem(spacerItem2)
                horizontalLayout.addWidget(widget)
                horizontalLayout.addWidget(img_label)
            else:
                horizontalLayout.addWidget(img_label)
                horizontalLayout.addWidget(widget)
                horizontalLayout.addItem(spacerItem2)
                

            horizontalLayout.setStretch(0, 1)
            horizontalLayout.setStretch(1, 3)
            horizontalLayout.setStretch(2, 1)
            self.scrollArea_layout.addLayout(horizontalLayout)
        self.scrollAreaWidgetContents.setGeometry(0,0,510,len(self.messages*55))

    def clickChat(self):
        print(1)


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

    def clearLayout(self, layout):
        item_list = list(range(layout.count()))
        item_list.reverse()

        for i in item_list:
            item = layout.itemAt(i)
            layout.removeItem(item)
            if item.widget():
                item.widget().deleteLater()
            else:
                self.clearLayout(item)
