# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file './ui_src/register.ui'
#
# Created by: PyQt5 UI code generator 5.9.2
#
# WARNING! All changes made in this file will be lost!

from PyQt5 import QtCore, QtGui, QtWidgets

class Ui_MainWindow(object):
    def setupUi(self, MainWindow):
        MainWindow.setObjectName("MainWindow")
        MainWindow.resize(460, 300)
        self.centralwidget = QtWidgets.QWidget(MainWindow)
        self.centralwidget.setObjectName("centralwidget")
        self.gif = QtWidgets.QTextBrowser(self.centralwidget)
        self.gif.setGeometry(QtCore.QRect(0, 0, 460, 130))
        self.gif.setObjectName("gif")
        self.nick = QtWidgets.QLineEdit(self.centralwidget)
        self.nick.setGeometry(QtCore.QRect(110, 140, 241, 31))
        self.nick.setInputMask("")
        self.nick.setText("")
        self.nick.setObjectName("nick")
        self.password = QtWidgets.QLineEdit(self.centralwidget)
        self.password.setGeometry(QtCore.QRect(110, 180, 241, 31))
        self.password.setEchoMode(QtWidgets.QLineEdit.Password)
        self.password.setObjectName("password")
        self.login = QtWidgets.QPushButton(self.centralwidget)
        self.login.setGeometry(QtCore.QRect(190, 270, 75, 23))
        self.login.setObjectName("login")
        self.showPassword = QtWidgets.QCheckBox(self.centralwidget)
        self.showPassword.setGeometry(QtCore.QRect(270, 260, 71, 16))
        self.showPassword.setObjectName("showPassword")
        self.passwordAgain = QtWidgets.QLineEdit(self.centralwidget)
        self.passwordAgain.setGeometry(QtCore.QRect(110, 220, 241, 31))
        self.passwordAgain.setEchoMode(QtWidgets.QLineEdit.Password)
        self.passwordAgain.setObjectName("passwordAgain")
        self.toLogin = QtWidgets.QToolButton(self.centralwidget)
        self.toLogin.setGeometry(QtCore.QRect(420, 280, 37, 18))
        self.toLogin.setObjectName("toLogin")
        MainWindow.setCentralWidget(self.centralwidget)

        self.retranslateUi(MainWindow)
        QtCore.QMetaObject.connectSlotsByName(MainWindow)

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
