# -*- coding: utf-8 -*-

from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtWidgets import QApplication,QMainWindow
import sys

class Ui_MainWindow(object):
    def __init__(self,client):
        super().__init__()
        self.client = None
        self.setClientSystem(client)
    
    def setClientSystem(self,system):
        self.client = system

    def setupUi(self, MainWindow):
        MainWindow.setObjectName("MainWindow")
        MainWindow.resize(1100, 600)

        icon = QtGui.QIcon()
        icon.addPixmap(QtGui.QPixmap("./resources/OurChat_Logo_low_pixel.png"), QtGui.QIcon.Normal, QtGui.QIcon.Off)
        MainWindow.setWindowIcon(icon)

        self.centralwidget = QtWidgets.QWidget(MainWindow)
        self.centralwidget.setObjectName("centralwidget")

        self.Logo = QtWidgets.QLabel(self.centralwidget)
        self.Logo.setGeometry(QtCore.QRect(10, 0, 61, 61))
        self.Logo.setObjectName("Logo")

        self.Chats = QtWidgets.QScrollArea(self.centralwidget)
        self.Chats.setGeometry(QtCore.QRect(80, 10, 231, 591))
        self.Chats.setWidgetResizable(True)
        self.Chats.setObjectName("Chats")

        self.scrollAreaWidgetContents = QtWidgets.QWidget()
        self.scrollAreaWidgetContents.setGeometry(QtCore.QRect(0, 0, 229, 589))
        self.scrollAreaWidgetContents.setObjectName("scrollAreaWidgetContents")

        self.Chats.setWidget(self.scrollAreaWidgetContents)

        self.ChatName = QtWidgets.QLabel(self.centralwidget)
        self.ChatName.setGeometry(QtCore.QRect(340, 0, 761, 41))
        self.ChatName.setObjectName("ChatName")

        self.messageBrowser = QtWidgets.QTextBrowser(self.centralwidget)
        self.messageBrowser.setGeometry(QtCore.QRect(340, 50, 741, 411))
        self.messageBrowser.setObjectName("messageBrowser")

        self.inputText = QtWidgets.QTextEdit(self.centralwidget)
        self.inputText.setGeometry(QtCore.QRect(340, 500, 741, 101))
        self.inputText.setObjectName("inputText")

        self.sendButton = QtWidgets.QPushButton(self.centralwidget)
        self.sendButton.setGeometry(QtCore.QRect(990, 570, 75, 23))
        self.sendButton.setObjectName("sendButton")

        self.remind = QtWidgets.QLabel(self.centralwidget)
        self.remind.setGeometry(QtCore.QRect(923, 480, 141, 20))
        self.remind.setObjectName("remind")

        self.emojiButton = QtWidgets.QPushButton(self.centralwidget)
        self.emojiButton.setGeometry(QtCore.QRect(350, 470, 31, 31))
        self.emojiButton.setObjectName("emojiButton")

        self.imageButton = QtWidgets.QPushButton(self.centralwidget)
        self.imageButton.setGeometry(QtCore.QRect(390, 470, 31, 31))
        self.imageButton.setObjectName("imageButton")

        self.SettingButton = QtWidgets.QPushButton(self.centralwidget)
        self.SettingButton.setGeometry(QtCore.QRect(0, 570, 75, 23))
        self.SettingButton.setObjectName("SettingButton")

        MainWindow.setCentralWidget(self.centralwidget)

        self.retranslateUi(MainWindow)
        self.bind()
        QtCore.QMetaObject.connectSlotsByName(MainWindow)
    
    def bind(self):
        self.sendButton.clicked.connect(self.sendNormorMessage)
    
    def setRemind(self,text,color="#000000"):
        self.remind.setText(f"<html><head/><body><p align=\"right\"><span style=\" color:{color};\">{text}</span></p></body></html>")
    
    def sendNormorMessage(self):
        msg = self.inputText.toPlainText()
        if msg.replace(" ","") == "": # 防止恶意刷屏
            self.setRemind("不能发送空白信息","#ff0000")
        else:
            # self.client.sendNormorMessage() # 调用客户端发送信息
            self.setRemind("")
            self.inputText.setPlainText("")
    
    def retranslateUi(self, MainWindow):
        _translate = QtCore.QCoreApplication.translate
        MainWindow.setWindowTitle(_translate("MainWindow", "OurChat"))
        self.Logo.setText(_translate("MainWindow", "<html><head/><body><p><img src=\"./resources/OurChat_Logo_low_pixel.png\"/></p></body></html>"))
        self.ChatName.setText(_translate("MainWindow", "<html><head/><body><p align=\"center\"><span style=\" font-size:18pt;\">聊天名字</span></p></body></html>"))
        self.sendButton.setText(_translate("MainWindow", "发送"))
        self.remind.setText(_translate("MainWindow", ""))
        self.emojiButton.setText(_translate("MainWindow", "表情"))
        self.imageButton.setText(_translate("MainWindow", "图片"))
        self.SettingButton.setText(_translate("MainWindow", "设置"))
    


if __name__ == "__main__":
    app = QApplication(sys.argv)
    mainWindow = QMainWindow()
    ui = Ui_MainWindow(None)
    ui.setupUi(mainWindow)
    mainWindow.show()
    sys.exit(app.exec_())