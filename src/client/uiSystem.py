from PyQt5 import QtCore, QtGui, QtWidgets

from PyQt5.QtWidgets import QApplication, QMainWindow
from PyQt5.QtCore import QTimer
from ui import main_window
import sys, os

class mainUi(main_window.Ui_MainWindow):
    def __init__(self, window):
        self.client = None
        self.chats = []
        self.lang = {}
        self.lang_file_name = ""
        self.config = {}
        self.window = window
        self.more_lang_index = 13
        self.to_another_text = 14
        self.login_button_text = 0
        self.login_account_text = 10
        self.func_queue = []
        self.tick()

    def retranslateUi(self, MainWindow):
        self.bind()
        self.UiLoadingDone()

    def renameUi(self):
        _translate = QtCore.QCoreApplication.translate
        self.window.setWindowTitle(_translate("MainWindow", "OurChat"))
        self.logo.setText(
            _translate(
                "MainWindow",
                '<html><head/><body><p><img src="../../resource/OurChat_Logo_low_pixel.png"/></p></body></html>',
            )
        )

        self.chatButton.setText(_translate("MainWindow", self.lang[9]))
        self.settingButton.setText(_translate("MainWindow", self.lang[2]))
        self.CantConnectServer.setText(
            _translate(
                "MainWindow",
                f'<html><head/><body><p align="center"><span style=" font-weight:600; color:#ff0000;">{self.lang[3]}</span></p></body></html>',
            )
        )
        self.chat_send.setText(_translate("MainWindow", self.lang[4]))
        self.chat_name.setText(
            _translate(
                "MainWindow",
                '<html><head/><body><p align="center"><span style=" font-size:28pt;">ChatName</span></p></body></html>',
            )
        )
        self.setting_Logo.setText(
            _translate(
                "MainWindow",
                '<html><head/><body><img src="../../resource/OurChat_logo.png"></body></html>',
            )
        )
        self.setting_server_ip_text.setText(_translate("MainWindow", self.lang[5]))
        self.setting_serverIp.setPlaceholderText(_translate("MainWindow", self.lang[5]))
        self.setting_lang_text.setText(_translate("MainWindow", self.lang[6]))
        self.setting_clearCache.setText(_translate("MainWindow", self.lang[7]))
        self.setting_logout.setText(_translate("MainWindow", self.lang[8]))
        self.login_startImg.setText(
            _translate(
                "MainWindow",
                '<html><head/><body><img src="../../resource/startImg.png" /></body></html>',
            )
        )
        self.login_account.setPlaceholderText(_translate("MainWindow", self.lang[self.login_account_text]))
        self.login_password.setEchoMode(QtWidgets.QLineEdit.Password)
        self.login_password.setPlaceholderText(_translate("MainWindow", self.lang[11]))
        self.login_login.setText(_translate("MainWindow", self.lang[self.login_button_text]))
        self.login_showPassword.setText(_translate("MainWindow", self.lang[12]))
        self.login_more.setText(_translate("MainWindow", self.lang[self.more_lang_index]))
        self.login_another.setText(_translate("MainWindow", self.lang[self.to_another_text]))
        self.login_serverIp.setPlaceholderText(_translate("MainWindow", self.lang[5]))
        self.login_lang_text.setText(_translate("MainWindow", self.lang[6]))

    def loadLang(self):
        self.lang = {}
        if self.setting_lang.currentText() == self.lang_file_name:
            file_name = self.login_lang.currentText()
        else:
            file_name = self.setting_lang.currentText()
        
        self.lang_file_name = file_name

        if file_name == "":
            return None

        with open(f"./lang/{file_name}", "r", encoding="utf-8") as f:
            for line in f.readlines():
                num, text = line.split("=")
                num = int(num)
                text = text.replace("\n", "")
                self.lang[num] = text

        self.config["lang_file"] = file_name
        self.client.setConfig(self.config)
        self.renameUi()

    def getServerIp(self):
        return self.setting_serverIp.text()

    def hideChat(self):
        self.chat_widget.hide()

    def showChat(self):
        self.chat_widget.show()
        self.hideSetting()
        self.hideLogin()
        self.window.resize(880, 620)

    def hideSetting(self):
        self.setting_widget.hide()

    def showSetting(self):
        self.window.resize(880, 620)
        self.updateLangFile()
        self.setting_widget.show()
        self.hideLogin()
        self.hideChat()
    
    def chatChange(self):
        self.client.readChatPartRecord(self.now_chatId,)

    def showLogin(self):
        self.window.resize(460, 350)
        self.login_more.setGeometry(QtCore.QRect(420, 330, 37, 18))
        self.login_widget.show()
        self.chat_widget.hide()
        self.setting_widget.hide()
    
    def clickLoginMore(self):
        if self.more_lang_index == 13:
            self.more_lang_index = 17
            self.loginOpenMore()
        else:
            self.more_lang_index = 13
            self.loginCloseMore()
    
    def loginOpenMore(self):
        _translate = QtCore.QCoreApplication.translate
        self.window.resize(460, 550)
        self.login_more.setGeometry(QtCore.QRect(420, 530, 37, 18))
        self.login_more.setText(_translate("MainWindow", self.lang[17]))
    
    def loginCloseMore(self):
        _translate = QtCore.QCoreApplication.translate
        self.window.resize(460, 350)
        self.login_more.setGeometry(QtCore.QRect(420, 330, 37, 18))
        self.login_more.setText(_translate("MainWindow", self.lang[13]))

    def hideLogin(self):
        self.login_widget.hide()

    def startUi(self):
        app = QApplication(sys.argv)
        window = QMainWindow()
        self.setupUi(window)
        window.show()
        app.exec_()

        self.client.ui = None

    def startGithub(self):
        os.system("start https://github.com/Yang-Lin-Team/OurChat")

    def login(self):
        if (
            self.login_account.text().replace(" ", "") == ""
            or self.login_password.text().replace(" ", "") == ""
        ):
            self.setLoginTip(self.lang[16])
            return None

        if self.login_button_text == 0:
            self.setLoginTip("")
            self.client.login(self.login_account.text(), self.login_password.text())
        else:
            self.setLoginTip("")
            self.client.register(self.login_account.text(), self.login_password.text())

    def bind(self):
        self.settingButton.clicked.connect(self.showSetting)
        self.chatButton.clicked.connect(self.showChat)
        self.setting_lang.currentTextChanged.connect(self.loadLang)
        self.login_lang.currentTextChanged.connect(self.loadLang)
        self.ServerIpChanged(self.config["server_ip"])
        self.setting_serverIp.textChanged.connect(self.ServerIpChanged)
        self.login_serverIp.textChanged.connect(self.ServerIpChanged)
        self.login_login.clicked.connect(self.login)
        self.login_showPassword.clicked.connect(self.passwordStateChangeed)
        self.login_more.clicked.connect(self.clickLoginMore)
        self.login_another.clicked.connect(self.changeLogin)
        self.chat_send.clicked.connect(self.sendNormalMsg)

    def passwordStateChangeed(self):
        if self.login_showPassword.isChecked():
            self.login_password.setEchoMode(QtWidgets.QLineEdit.Normal)
        else:
            self.login_password.setEchoMode(QtWidgets.QLineEdit.Password)
    
    def getOcid(self):
        return self.login_account.text()

    def setLoginTip(self,text):
        _translate = QtCore.QCoreApplication.translate
        self.login_tip.setText(_translate("MainWindow", f"<html><head/><body><p align=\"right\"><span style=\" color:#ff0000;\">{text}</span></p></body></html>"))
        self.login_tip.show()

    def ServerIpChanged(self, ip):
        self.client.setServerIp(ip)  # 传递给客户端
        self.config["server_ip"] = ip
        self.client.setConfig(self.config)  # 写入配置文件
        self.setting_serverIp.setText(ip)

    def setIpUiText(self, text):
        self.setting_serverIp.setText(text)
        self.login_serverIp.setText(text)
    
    def sendNormalMsg(self):
        print(self.chat_input.toPlainText())
        self.client.sendNormalMsg(1,self.chat_input.toPlainText())

    def UiLoadingDone(self):
        self.updateLangFile()
        self.loadLang()
        config_data = self.config
        self.setting_serverIp.setText(config_data["server_ip"])
        self.login_serverIp.setText(config_data["server_ip"])
        self.renameUi()

        self.showLogin()

        self.client.setUi(self)
        self.client.start()

    def setClientSystem(self, client):
        self.client = client
        self.config = self.client.getConfig()

    def updateLangFile(self):
        self.setting_lang.clear()

        # 先添加当前配置语言 否则为空时添加语言 改变事件会直接更改配置文件
        config_data = self.config
        self.setting_lang.addItem(config_data["lang_file"])
        self.setting_lang.setCurrentText(config_data["lang_file"])
        self.login_lang.addItem(config_data["lang_file"])
        self.login_lang.setCurrentText(config_data["lang_file"])

        for lang_file in os.listdir("lang"):
            if lang_file == config_data["lang_file"]:
                continue
            self.setting_lang.addItem(lang_file)
            self.login_lang.addItem(lang_file)
    
    def changeLogin(self):
        if self.to_another_text == 14:
            self.to_another_text = 15
            self.login_button_text = 1
            self.login_account_text = 18
            self.renameUi()
        else:
            self.to_another_text = 14
            self.login_button_text = 0
            self.login_account_text = 10
            self.renameUi()
    
    def tick(self):
        for func in self.func_queue:
            func()
            self.func_queue.remove(func)

        self.timer = QTimer()
        self.timer.timeout.connect(self.tick)
        self.timer.start(25)

class UiCotrol:
    def __init__(self, client):
        app = QApplication(sys.argv)
        window = QMainWindow()

        a = mainUi(window)
        a.setClientSystem(client)

        a.setupUi(window)
        window.show()
        app.exec_()
        a.client.ui = None
