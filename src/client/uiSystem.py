from PyQt5 import QtCore, QtGui, QtWidgets
from PyQt5.QtWidgets import QApplication,QMainWindow
import sys,os
from ui import main_window,register,login

class RegisterUi(register.Ui_MainWindow):
    def __init__(self):
        pass    
    
    def retranslateUi(self, MainWindow):
        _translate = QtCore.QCoreApplication.translate
        MainWindow.setWindowTitle(_translate("MainWindow", "MainWindow"))
        self.gif.setHtml(_translate("MainWindow", "<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.0//EN\" \"http://www.w3.org/TR/REC-html40/strict.dtd\">\n""<html><head><meta name=\"qrichtext\" content=\"1\" /><style type=\"text/css\">\n""p, li { white-space: pre-wrap; }\n""</style></head><body style=\" font-family:\'SimSun\'; font-size:9pt; font-weight:400; font-style:normal;\">\n""<p style=\" margin-top:0px; margin-bottom:0px; margin-left:0px; margin-right:0px; -qt-block-indent:0; text-indent:0px;\">这里是个gif</p></body></html>"))
        self.nick.setPlaceholderText(_translate("MainWindow", "昵称"))
        self.password.setPlaceholderText(_translate("MainWindow", "密码"))
        self.login.setText(_translate("MainWindow", "注册"))
        self.showPassword.setText(_translate("MainWindow", "显示密码"))
        self.passwordAgain.setPlaceholderText(_translate("MainWindow", "再次输入密码"))
        self.toLogin.setText(_translate("MainWindow", "登录"))

        self.bind()
    
    def changePasswordState(self,state):
        if state:
            password_state = QtWidgets.QLineEdit.Normal
        else:
            password_state = QtWidgets.QLineEdit.Password

        self.password.setEchoMode(password_state)
        self.passwordAgain.setEchoMode(password_state)
    
    def bind(self):
        self.showPassword.clicked.connect(self.changePasswordState)

    def startUi(self):
        app = QApplication(sys.argv)
        window = QMainWindow()
        self.setupUi(window)
        window.show()
        app.exec_()

class mainUi(main_window.Ui_MainWindow):
    def __init__(self):
        self.client = None
        self.chats = []
    
    def retranslateUi(self, MainWindow):
        _translate = QtCore.QCoreApplication.translate
        MainWindow.setWindowTitle(_translate("MainWindow", "OurChat"))
        self.logo.setText(_translate("MainWindow", "<html><head/><body><p><img src=\"../../resource/OurChat_Logo_low_pixel.png\"/></p></body></html>"))
        self.chatButton.setText(_translate("MainWindow", "聊天"))
        self.settingButton.setText(_translate("MainWindow", "设置"))
        self.chat_name.setText(_translate("MainWindow", "<html><head/><body><p align=\"center\"><span style=\" font-size:28pt;\">ChatName</span></p></body></html>"))
        self.chat_send.setText(_translate("MainWindow", "发送"))

        self.setting_logo.setText(_translate("MainWindow", "<html><head/><body><p><img src=\"../../resource/OurChat_logo.png\"/></p></body></html>"))
        self.setting_clearCache.setText(_translate("MainWindow", "清空缓存"))
        self.setting_text.setText(_translate("MainWindow", "服务器IP"))
        self.setting_toGithub.setText(_translate("MainWindow", "Github地址"))
        self.setting_serverIp.setText("127.0.0.1")
        self.CantConnectServer.setText(_translate("MainWindow", "<html><head/><body><p><span style=\" font-weight:600; color:#ff0000;\">无法连接至服务器</span></p></body></html>"))

        self.bind()
        self.hideSetting()

        self.UiLoadingDone()
    
    def updateUiChats(self):
        self.scrollAreaWidgetContents.setGeometry(QtCore.QRect(0, 0, 180, max(561,len(self.chats)*50+5)))
    
    def getServerIp(self):
        return self.setting_serverIp.text()

    def hideChat(self):
        self.chat_chats.hide()
        self.chat_brower.hide()
        self.chat_input.hide()
        self.chat_emojiButton.hide()
        self.chat_imageButton.hide()
        self.chat_send.hide()
        self.chat_search.hide()
        self.chat_name.hide()
        self.chat_fileButton.hide()

    def showChat(self):
        self.chat_chats.show()
        self.chat_brower.show()
        self.chat_input.show()
        self.chat_emojiButton.show()
        self.chat_imageButton.show()
        self.chat_send.show()
        self.chat_search.show()
        self.chat_name.show()
        self.chat_fileButton.show()
        self.hideSetting()

    def hideSetting(self):
        self.setting_logo.hide()
        self.setting_clearCache.hide()
        self.setting_text.hide()
        self.setting_toGithub.hide()
        self.setting_serverIp.hide()
    
    def showSetting(self):
        self.setting_logo.show()
        self.setting_clearCache.show()
        self.setting_text.show()
        self.setting_toGithub.show()
        self.setting_serverIp.show()
        self.hideChat()
    
    def startUi(self):
        app = QApplication(sys.argv)
        window = QMainWindow()
        self.setupUi(window)
        window.show()
        app.exec_()
        self.client.ui = None
    
    def startGithub(self):
        os.system("start https://github.com/Yang-Lin-Team/OurChat")
    
    def bind(self):
        self.settingButton.clicked.connect(self.showSetting)
        self.chatButton.clicked.connect(self.showChat)
        self.client.setServerIp(self.getServerIp())
        self.setting_serverIp.textChanged.connect(self.client.setServerIp)
        self.setting_toGithub.clicked.connect(self.startGithub)
    
    def UiLoadingDone(self):
        self.client.start()
    
    def setClientSystem(self,client):
        self.client = client

class UiCotrol():
    def __init__(self,client):
        app = QApplication(sys.argv)
        window = QMainWindow()
        '''a = RegisterUi() # 生成注册窗口
        a.setupUi(window)
        window.show()
        app.exec_()'''

        a = mainUi()
        a.setClientSystem(client) # 生成主窗口
        a.setupUi(window)
        window.show()
        app.exec_()
        a.client.ui = None
