# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file 'd:\OurChat\client\pc\ui\main.ui'
#
# Created by: PyQt5 UI code generator 5.15.9
#
# WARNING: Any manual changes made to this file will be lost when pyuic5 is
# run again.  Do not edit this file unless you know what you are doing.


from PyQt5 import QtCore, QtGui, QtWidgets


class Ui_Main(object):
    def setupUi(self, Main):
        Main.setObjectName("Main")
        Main.resize(800, 600)
        Main.setWindowTitle("MainWindow")
        self.centralwidget = QtWidgets.QWidget(Main)
        self.centralwidget.setObjectName("centralwidget")
        self.horizontalLayout = QtWidgets.QHBoxLayout(self.centralwidget)
        self.horizontalLayout.setObjectName("horizontalLayout")
        self.verticalLayout = QtWidgets.QVBoxLayout()
        self.verticalLayout.setObjectName("verticalLayout")
        self.to_chat = QtWidgets.QPushButton(self.centralwidget)
        self.to_chat.setText("Chat")
        self.to_chat.setObjectName("to_chat")
        self.verticalLayout.addWidget(self.to_chat)
        self.pushButton_3 = QtWidgets.QPushButton(self.centralwidget)
        self.pushButton_3.setText("Account")
        self.pushButton_3.setObjectName("pushButton_3")
        self.verticalLayout.addWidget(self.pushButton_3)
        self.to_setting = QtWidgets.QPushButton(self.centralwidget)
        self.to_setting.setText("Setting")
        self.to_setting.setObjectName("to_setting")
        self.verticalLayout.addWidget(self.to_setting)
        spacerItem = QtWidgets.QSpacerItem(20, 40, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding)
        self.verticalLayout.addItem(spacerItem)
        self.horizontalLayout.addLayout(self.verticalLayout)
        self.widget = QtWidgets.QWidget(self.centralwidget)
        self.widget.setObjectName("widget")
        self.horizontalLayout.addWidget(self.widget)
        self.horizontalLayout.setStretch(0, 1)
        self.horizontalLayout.setStretch(1, 9)
        Main.setCentralWidget(self.centralwidget)
        self.statusbar = QtWidgets.QStatusBar(Main)
        self.statusbar.setObjectName("statusbar")
        Main.setStatusBar(self.statusbar)

        self.retranslateUi(Main)
        QtCore.QMetaObject.connectSlotsByName(Main)

    def retranslateUi(self, Main):
        pass
