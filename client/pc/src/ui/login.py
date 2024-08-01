# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file 'login.ui'
#
# Created by: PyQt5 UI code generator 5.15.9
#
# WARNING: Any manual changes made to this file will be lost when pyuic5 is
# run again.  Do not edit this file unless you know what you are doing.


from PyQt5 import QtCore, QtWidgets


class Ui_Login(object):
    def setupUi(self, Login):
        Login.setObjectName("Login")
        Login.resize(370, 250)
        self.horizontalLayout_3 = QtWidgets.QHBoxLayout(Login)
        self.horizontalLayout_3.setObjectName("horizontalLayout_3")
        spacerItem = QtWidgets.QSpacerItem(
            21, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_3.addItem(spacerItem)
        self.verticalLayout = QtWidgets.QVBoxLayout()
        self.verticalLayout.setObjectName("verticalLayout")
        self.tabWidget = QtWidgets.QTabWidget(Login)
        self.tabWidget.setObjectName("tabWidget")
        self.login_tab = QtWidgets.QWidget()
        self.login_tab.setObjectName("login_tab")
        self.formLayout = QtWidgets.QFormLayout(self.login_tab)
        self.formLayout.setObjectName("formLayout")
        spacerItem1 = QtWidgets.QSpacerItem(
            20, 40, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.formLayout.setItem(0, QtWidgets.QFormLayout.FieldRole, spacerItem1)
        self.label_3 = QtWidgets.QLabel(self.login_tab)
        self.label_3.setText("OCID/Email")
        self.label_3.setObjectName("label_3")
        self.formLayout.setWidget(1, QtWidgets.QFormLayout.LabelRole, self.label_3)
        self.login_account_editor = QtWidgets.QLineEdit(self.login_tab)
        self.login_account_editor.setObjectName("login_account_editor")
        self.formLayout.setWidget(
            1, QtWidgets.QFormLayout.FieldRole, self.login_account_editor
        )
        self.label_4 = QtWidgets.QLabel(self.login_tab)
        self.label_4.setText("Password")
        self.label_4.setObjectName("label_4")
        self.formLayout.setWidget(2, QtWidgets.QFormLayout.LabelRole, self.label_4)
        self.login_password_editor = QtWidgets.QLineEdit(self.login_tab)
        self.login_password_editor.setEchoMode(QtWidgets.QLineEdit.Password)
        self.login_password_editor.setObjectName("login_password_editor")
        self.formLayout.setWidget(
            2, QtWidgets.QFormLayout.FieldRole, self.login_password_editor
        )
        self.horizontalLayout = QtWidgets.QHBoxLayout()
        self.horizontalLayout.setObjectName("horizontalLayout")
        spacerItem2 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout.addItem(spacerItem2)
        self.login_show_checkbox = QtWidgets.QCheckBox(self.login_tab)
        self.login_show_checkbox.setText("Show")
        self.login_show_checkbox.setObjectName("login_show_checkbox")
        self.horizontalLayout.addWidget(self.login_show_checkbox)
        self.formLayout.setLayout(
            3, QtWidgets.QFormLayout.FieldRole, self.horizontalLayout
        )
        self.tabWidget.addTab(self.login_tab, "Login")
        self.register_tab = QtWidgets.QWidget()
        self.register_tab.setObjectName("register_tab")
        self.formLayout_2 = QtWidgets.QFormLayout(self.register_tab)
        self.formLayout_2.setObjectName("formLayout_2")
        spacerItem3 = QtWidgets.QSpacerItem(
            20, 40, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.formLayout_2.setItem(0, QtWidgets.QFormLayout.FieldRole, spacerItem3)
        self.label_5 = QtWidgets.QLabel(self.register_tab)
        self.label_5.setText("Email")
        self.label_5.setObjectName("label_5")
        self.formLayout_2.setWidget(1, QtWidgets.QFormLayout.LabelRole, self.label_5)
        self.register_email_editor = QtWidgets.QLineEdit(self.register_tab)
        self.register_email_editor.setObjectName("register_email_editor")
        self.formLayout_2.setWidget(
            1, QtWidgets.QFormLayout.FieldRole, self.register_email_editor
        )
        self.label_6 = QtWidgets.QLabel(self.register_tab)
        self.label_6.setText("Password")
        self.label_6.setObjectName("label_6")
        self.formLayout_2.setWidget(2, QtWidgets.QFormLayout.LabelRole, self.label_6)
        self.register_password_editor = QtWidgets.QLineEdit(self.register_tab)
        self.register_password_editor.setEchoMode(QtWidgets.QLineEdit.Password)
        self.register_password_editor.setObjectName("register_password_editor")
        self.formLayout_2.setWidget(
            2, QtWidgets.QFormLayout.FieldRole, self.register_password_editor
        )
        self.horizontalLayout_2 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_2.setObjectName("horizontalLayout_2")
        spacerItem4 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_2.addItem(spacerItem4)
        self.register_show_checkbox = QtWidgets.QCheckBox(self.register_tab)
        self.register_show_checkbox.setText("Show")
        self.register_show_checkbox.setObjectName("register_show_checkbox")
        self.horizontalLayout_2.addWidget(self.register_show_checkbox)
        self.formLayout_2.setLayout(
            3, QtWidgets.QFormLayout.FieldRole, self.horizontalLayout_2
        )
        self.tabWidget.addTab(self.register_tab, "Register")
        self.verticalLayout.addWidget(self.tabWidget)
        self.horizontalLayout_4 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_4.setObjectName("horizontalLayout_4")
        spacerItem5 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_4.addItem(spacerItem5)
        self.label = QtWidgets.QLabel(Login)
        self.label.setText("Server   IP")
        self.label.setObjectName("label")
        self.horizontalLayout_4.addWidget(self.label)
        self.server_ip_editor = QtWidgets.QLineEdit(Login)
        self.server_ip_editor.setObjectName("server_ip_editor")
        self.horizontalLayout_4.addWidget(self.server_ip_editor)
        spacerItem6 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_4.addItem(spacerItem6)
        self.verticalLayout.addLayout(self.horizontalLayout_4)
        self.horizontalLayout_5 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_5.setObjectName("horizontalLayout_5")
        spacerItem7 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_5.addItem(spacerItem7)
        self.label_2 = QtWidgets.QLabel(Login)
        self.label_2.setText("Server Port")
        self.label_2.setObjectName("label_2")
        self.horizontalLayout_5.addWidget(self.label_2)
        self.server_port_editor = QtWidgets.QLineEdit(Login)
        self.server_port_editor.setText("")
        self.server_port_editor.setObjectName("server_port_editor")
        self.horizontalLayout_5.addWidget(self.server_port_editor)
        spacerItem8 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_5.addItem(spacerItem8)
        self.verticalLayout.addLayout(self.horizontalLayout_5)
        self.horizontalLayout_6 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_6.setObjectName("horizontalLayout_6")
        spacerItem9 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_6.addItem(spacerItem9)
        self.connect_server_btn = QtWidgets.QPushButton(Login)
        self.connect_server_btn.setText("Connect server")
        self.connect_server_btn.setObjectName("connect_server_btn")
        self.horizontalLayout_6.addWidget(self.connect_server_btn)
        self.join_btn = QtWidgets.QPushButton(Login)
        self.join_btn.setText("Join")
        self.join_btn.setObjectName("join_btn")
        self.horizontalLayout_6.addWidget(self.join_btn)
        spacerItem10 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_6.addItem(spacerItem10)
        self.verticalLayout.addLayout(self.horizontalLayout_6)
        self.horizontalLayout_3.addLayout(self.verticalLayout)
        spacerItem11 = QtWidgets.QSpacerItem(
            25, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_3.addItem(spacerItem11)

        self.retranslateUi(Login)
        self.tabWidget.setCurrentIndex(0)
        QtCore.QMetaObject.connectSlotsByName(Login)

    def retranslateUi(self, Login):
        _translate = QtCore.QCoreApplication.translate
        Login.setWindowTitle(_translate("Login", "Form"))
