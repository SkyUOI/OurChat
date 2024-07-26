# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file 'd:\OurChat\client\pc\ui\chat.ui'
#
# Created by: PyQt5 UI code generator 5.15.9
#
# WARNING: Any manual changes made to this file will be lost when pyuic5 is
# run again.  Do not edit this file unless you know what you are doing.


from PyQt5 import QtCore, QtGui, QtWidgets


class Ui_Form(object):
    def setupUi(self, Form):
        Form.setObjectName("Form")
        Form.resize(400, 300)
        self.horizontalLayout = QtWidgets.QHBoxLayout(Form)
        self.horizontalLayout.setObjectName("horizontalLayout")
        self.verticalLayout_3 = QtWidgets.QVBoxLayout()
        self.verticalLayout_3.setObjectName("verticalLayout_3")
        self.search_chat = QtWidgets.QLineEdit(Form)
        self.search_chat.setObjectName("search_chat")
        self.verticalLayout_3.addWidget(self.search_chat)
        self.chat_list = QtWidgets.QScrollArea(Form)
        self.chat_list.setHorizontalScrollBarPolicy(QtCore.Qt.ScrollBarAlwaysOff)
        self.chat_list.setWidgetResizable(False)
        self.chat_list.setObjectName("chat_list")
        self.scrollAreaWidgetContents = QtWidgets.QWidget()
        self.scrollAreaWidgetContents.setGeometry(QtCore.QRect(0, 0, 200, 60))
        self.scrollAreaWidgetContents.setObjectName("scrollAreaWidgetContents")
        self.verticalLayout_2 = QtWidgets.QVBoxLayout(self.scrollAreaWidgetContents)
        self.verticalLayout_2.setObjectName("verticalLayout_2")
        self.horizontalLayout_4 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_4.setObjectName("horizontalLayout_4")
        self.label = QtWidgets.QLabel(self.scrollAreaWidgetContents)
        self.label.setText("img")
        self.label.setObjectName("label")
        self.horizontalLayout_4.addWidget(self.label)
        self.verticalLayout = QtWidgets.QVBoxLayout()
        self.verticalLayout.setObjectName("verticalLayout")
        self.label_2 = QtWidgets.QLabel(self.scrollAreaWidgetContents)
        self.label_2.setText("name")
        self.label_2.setObjectName("label_2")
        self.verticalLayout.addWidget(self.label_2)
        self.label_3 = QtWidgets.QLabel(self.scrollAreaWidgetContents)
        self.label_3.setText("recent_message")
        self.label_3.setObjectName("label_3")
        self.verticalLayout.addWidget(self.label_3)
        self.verticalLayout.setStretch(0, 2)
        self.verticalLayout.setStretch(1, 1)
        self.horizontalLayout_4.addLayout(self.verticalLayout)
        self.horizontalLayout_4.setStretch(0, 2)
        self.horizontalLayout_4.setStretch(1, 7)
        self.verticalLayout_2.addLayout(self.horizontalLayout_4)
        self.chat_list.setWidget(self.scrollAreaWidgetContents)
        self.verticalLayout_3.addWidget(self.chat_list)
        self.horizontalLayout.addLayout(self.verticalLayout_3)
        self.verticalLayout_5 = QtWidgets.QVBoxLayout()
        self.verticalLayout_5.setObjectName("verticalLayout_5")
        self.horizontalLayout_2 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_2.setObjectName("horizontalLayout_2")
        spacerItem = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum)
        self.horizontalLayout_2.addItem(spacerItem)
        self.title = QtWidgets.QLabel(Form)
        self.title.setText("Title")
        self.title.setObjectName("title")
        self.horizontalLayout_2.addWidget(self.title)
        spacerItem1 = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum)
        self.horizontalLayout_2.addItem(spacerItem1)
        self.verticalLayout_5.addLayout(self.horizontalLayout_2)
        self.record = QtWidgets.QScrollArea(Form)
        self.record.setWidgetResizable(True)
        self.record.setObjectName("record")
        self.scrollAreaWidgetContents_2 = QtWidgets.QWidget()
        self.scrollAreaWidgetContents_2.setGeometry(QtCore.QRect(0, 0, 211, 132))
        self.scrollAreaWidgetContents_2.setObjectName("scrollAreaWidgetContents_2")
        self.record.setWidget(self.scrollAreaWidgetContents_2)
        self.verticalLayout_5.addWidget(self.record)
        self.editor = QtWidgets.QTextEdit(Form)
        self.editor.setObjectName("editor")
        self.verticalLayout_5.addWidget(self.editor)
        self.horizontalLayout_3 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_3.setObjectName("horizontalLayout_3")
        spacerItem2 = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum)
        self.horizontalLayout_3.addItem(spacerItem2)
        self.send_btn = QtWidgets.QPushButton(Form)
        self.send_btn.setText("send")
        self.send_btn.setObjectName("send_btn")
        self.horizontalLayout_3.addWidget(self.send_btn)
        self.verticalLayout_5.addLayout(self.horizontalLayout_3)
        self.verticalLayout_5.setStretch(0, 1)
        self.verticalLayout_5.setStretch(1, 10)
        self.verticalLayout_5.setStretch(2, 6)
        self.verticalLayout_5.setStretch(3, 1)
        self.horizontalLayout.addLayout(self.verticalLayout_5)

        self.retranslateUi(Form)
        QtCore.QMetaObject.connectSlotsByName(Form)

    def retranslateUi(self, Form):
        _translate = QtCore.QCoreApplication.translate
        Form.setWindowTitle(_translate("Form", "Form"))
