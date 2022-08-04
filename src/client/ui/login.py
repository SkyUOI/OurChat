# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file 'ui_src/login.ui'
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
        self.id = QtWidgets.QLineEdit(self.centralwidget)
        self.id.setGeometry(QtCore.QRect(110, 160, 241, 31))
        self.id.setInputMask("")
        self.id.setText("")
        self.id.setObjectName("id")
        self.password = QtWidgets.QLineEdit(self.centralwidget)
        self.password.setGeometry(QtCore.QRect(110, 200, 241, 31))
        self.password.setEchoMode(QtWidgets.QLineEdit.Password)
        self.password.setObjectName("password")
        self.loginButton = QtWidgets.QPushButton(self.centralwidget)
        self.loginButton.setGeometry(QtCore.QRect(190, 260, 75, 23))
        self.loginButton.setObjectName("loginButton")
        self.showPassword = QtWidgets.QCheckBox(self.centralwidget)
        self.showPassword.setGeometry(QtCore.QRect(270, 240, 71, 16))
        self.showPassword.setObjectName("showPassword")
        self.toRegister = QtWidgets.QToolButton(self.centralwidget)
        self.toRegister.setGeometry(QtCore.QRect(420, 280, 37, 18))
        self.toRegister.setObjectName("toRegister")
        MainWindow.setCentralWidget(self.centralwidget)

        self.retranslateUi(MainWindow)
        QtCore.QMetaObject.connectSlotsByName(MainWindow)

    def retranslateUi(self, MainWindow):
        _translate = QtCore.QCoreApplication.translate
        MainWindow.setWindowTitle(_translate("MainWindow", "MainWindow"))
        self.gif.setHtml(_translate("MainWindow", "<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.0//EN\" \"http://www.w3.org/TR/REC-html40/strict.dtd\">\n""<html><head><meta name=\"qrichtext\" content=\"1\" /><style type=\"text/css\">\n""p, li { white-space: pre-wrap; }\n""</style></head><body style=\" font-family:\'SimSun\'; font-size:9pt; font-weight:400; font-style:normal;\">\n""<p style=\" margin-top:0px; margin-bottom:0px; margin-left:0px; margin-right:0px; -qt-block-indent:0; text-indent:0px;\">这里是个gif</p></body></html>"))
        self.id.setPlaceholderText(_translate("MainWindow", "账号"))
        self.password.setPlaceholderText(_translate("MainWindow", "密码"))
        self.loginButton.setText(_translate("MainWindow", "登录"))
        self.showPassword.setText(_translate("MainWindow", "显示密码"))
        self.toRegister.setText(_translate("MainWindow", "注册"))