# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file 'setting.ui'
#
# Created by: PyQt5 UI code generator 5.15.9
#
# WARNING: Any manual changes made to this file will be lost when pyuic5 is
# run again.  Do not edit this file unless you know what you are doing.


from PyQt5 import QtCore, QtWidgets


class Ui_Setting(object):
    def setupUi(self, Setting):
        Setting.setObjectName("Setting")
        Setting.resize(400, 300)
        Setting.setWindowTitle("Form")
        self.verticalLayout_2 = QtWidgets.QVBoxLayout(Setting)
        self.verticalLayout_2.setObjectName("verticalLayout_2")
        self.tabWidget = QtWidgets.QTabWidget(Setting)
        self.tabWidget.setObjectName("tabWidget")
        self.server = QtWidgets.QWidget()
        self.server.setObjectName("server")
        self.verticalLayout = QtWidgets.QVBoxLayout(self.server)
        self.verticalLayout.setObjectName("verticalLayout")
        spacerItem = QtWidgets.QSpacerItem(
            20, 57, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout.addItem(spacerItem)
        self.horizontalLayout = QtWidgets.QHBoxLayout()
        self.horizontalLayout.setObjectName("horizontalLayout")
        spacerItem1 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout.addItem(spacerItem1)
        self.formLayout = QtWidgets.QFormLayout()
        self.formLayout.setObjectName("formLayout")
        self.label = QtWidgets.QLabel(self.server)
        self.label.setText("IP")
        self.label.setObjectName("label")
        self.formLayout.setWidget(0, QtWidgets.QFormLayout.LabelRole, self.label)
        self.ip_editor = QtWidgets.QLineEdit(self.server)
        self.ip_editor.setObjectName("ip_editor")
        self.formLayout.setWidget(0, QtWidgets.QFormLayout.FieldRole, self.ip_editor)
        self.label_2 = QtWidgets.QLabel(self.server)
        self.label_2.setText("Port")
        self.label_2.setObjectName("label_2")
        self.formLayout.setWidget(1, QtWidgets.QFormLayout.LabelRole, self.label_2)
        self.port_editor = QtWidgets.QLineEdit(self.server)
        self.port_editor.setObjectName("port_editor")
        self.formLayout.setWidget(1, QtWidgets.QFormLayout.FieldRole, self.port_editor)
        self.label_3 = QtWidgets.QLabel(self.server)
        self.label_3.setText("Reconnection Attempt")
        self.label_3.setObjectName("label_3")
        self.formLayout.setWidget(2, QtWidgets.QFormLayout.LabelRole, self.label_3)
        self.reconnection_attempt_box = QtWidgets.QSpinBox(self.server)
        self.reconnection_attempt_box.setMinimum(1)
        self.reconnection_attempt_box.setMaximum(100)
        self.reconnection_attempt_box.setObjectName("reconnection_attempt_box")
        self.formLayout.setWidget(
            2, QtWidgets.QFormLayout.FieldRole, self.reconnection_attempt_box
        )
        self.horizontalLayout.addLayout(self.formLayout)
        spacerItem2 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout.addItem(spacerItem2)
        self.verticalLayout.addLayout(self.horizontalLayout)
        spacerItem3 = QtWidgets.QSpacerItem(
            20, 57, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout.addItem(spacerItem3)
        self.tabWidget.addTab(self.server, "Server")
        self.general = QtWidgets.QWidget()
        self.general.setObjectName("general")
        self.verticalLayout_3 = QtWidgets.QVBoxLayout(self.general)
        self.verticalLayout_3.setObjectName("verticalLayout_3")
        spacerItem4 = QtWidgets.QSpacerItem(
            20, 70, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout_3.addItem(spacerItem4)
        self.horizontalLayout_3 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_3.setObjectName("horizontalLayout_3")
        spacerItem5 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_3.addItem(spacerItem5)
        self.formLayout_2 = QtWidgets.QFormLayout()
        self.formLayout_2.setObjectName("formLayout_2")
        self.label_4 = QtWidgets.QLabel(self.general)
        self.label_4.setText("Language")
        self.label_4.setObjectName("label_4")
        self.formLayout_2.setWidget(0, QtWidgets.QFormLayout.LabelRole, self.label_4)
        self.language_combobox = QtWidgets.QComboBox(self.general)
        self.language_combobox.setObjectName("language_combobox")
        self.formLayout_2.setWidget(
            0, QtWidgets.QFormLayout.FieldRole, self.language_combobox
        )
        self.label_5 = QtWidgets.QLabel(self.general)
        self.label_5.setText("Theme")
        self.label_5.setObjectName("label_5")
        self.formLayout_2.setWidget(1, QtWidgets.QFormLayout.LabelRole, self.label_5)
        self.theme_combobox = QtWidgets.QComboBox(self.general)
        self.theme_combobox.setObjectName("theme_combobox")
        self.formLayout_2.setWidget(
            1, QtWidgets.QFormLayout.FieldRole, self.theme_combobox
        )
        self.horizontalLayout_3.addLayout(self.formLayout_2)
        spacerItem6 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_3.addItem(spacerItem6)
        self.verticalLayout_3.addLayout(self.horizontalLayout_3)
        spacerItem7 = QtWidgets.QSpacerItem(
            20, 70, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout_3.addItem(spacerItem7)
        self.tabWidget.addTab(self.general, "General")
        self.advanced = QtWidgets.QWidget()
        self.advanced.setObjectName("advanced")
        self.verticalLayout_4 = QtWidgets.QVBoxLayout(self.advanced)
        self.verticalLayout_4.setObjectName("verticalLayout_4")
        spacerItem8 = QtWidgets.QSpacerItem(
            20, 69, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout_4.addItem(spacerItem8)
        self.horizontalLayout_6 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_6.setObjectName("horizontalLayout_6")
        spacerItem9 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_6.addItem(spacerItem9)
        self.formLayout_3 = QtWidgets.QFormLayout()
        self.formLayout_3.setObjectName("formLayout_3")
        self.label_6 = QtWidgets.QLabel(self.advanced)
        self.label_6.setText("Log Level")
        self.label_6.setObjectName("label_6")
        self.formLayout_3.setWidget(0, QtWidgets.QFormLayout.LabelRole, self.label_6)
        self.log_level_combobox = QtWidgets.QComboBox(self.advanced)
        self.log_level_combobox.setObjectName("log_level_combobox")
        self.formLayout_3.setWidget(
            0, QtWidgets.QFormLayout.FieldRole, self.log_level_combobox
        )
        self.label_7 = QtWidgets.QLabel(self.advanced)
        self.label_7.setText("Log Saving Limit")
        self.label_7.setObjectName("label_7")
        self.formLayout_3.setWidget(1, QtWidgets.QFormLayout.LabelRole, self.label_7)
        self.horizontalLayout_4 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_4.setObjectName("horizontalLayout_4")
        self.log_saving_combobox = QtWidgets.QComboBox(self.advanced)
        self.log_saving_combobox.setObjectName("log_saving_combobox")
        self.horizontalLayout_4.addWidget(self.log_saving_combobox)
        self.label_8 = QtWidgets.QLabel(self.advanced)
        self.label_8.setText("Day(s)")
        self.label_8.setObjectName("label_8")
        self.horizontalLayout_4.addWidget(self.label_8)
        self.formLayout_3.setLayout(
            1, QtWidgets.QFormLayout.FieldRole, self.horizontalLayout_4
        )
        self.horizontalLayout_6.addLayout(self.formLayout_3)
        spacerItem10 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_6.addItem(spacerItem10)
        self.verticalLayout_4.addLayout(self.horizontalLayout_6)
        spacerItem11 = QtWidgets.QSpacerItem(
            20, 69, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding
        )
        self.verticalLayout_4.addItem(spacerItem11)
        self.tabWidget.addTab(self.advanced, "Advanced")
        self.about = QtWidgets.QWidget()
        self.about.setObjectName("about")
        self.verticalLayout_7 = QtWidgets.QVBoxLayout(self.about)
        self.verticalLayout_7.setObjectName("verticalLayout_7")
        self.horizontalLayout_5 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_5.setObjectName("horizontalLayout_5")
        self.logo_label = QtWidgets.QLabel(self.about)
        self.logo_label.setText("LOGO")
        self.logo_label.setObjectName("logo_label")
        self.horizontalLayout_5.addWidget(self.logo_label)
        self.gridLayout = QtWidgets.QGridLayout()
        self.gridLayout.setObjectName("gridLayout")
        self.version_label = QtWidgets.QLabel(self.about)
        self.version_label.setText("Version")
        self.version_label.setObjectName("version_label")
        self.gridLayout.addWidget(self.version_label, 0, 0, 1, 1)
        self.license_label = QtWidgets.QLabel(self.about)
        self.license_label.setText("License")
        self.license_label.setObjectName("license_label")
        self.gridLayout.addWidget(self.license_label, 0, 1, 1, 1)
        self.pushButton_3 = QtWidgets.QPushButton(self.about)
        self.pushButton_3.setObjectName("pushButton_3")
        self.gridLayout.addWidget(self.pushButton_3, 1, 0, 1, 1)
        self.horizontalLayout_5.addLayout(self.gridLayout)
        self.verticalLayout_7.addLayout(self.horizontalLayout_5)
        self.horizontalLayout_7 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_7.setObjectName("horizontalLayout_7")
        self.verticalLayout_5 = QtWidgets.QVBoxLayout()
        self.verticalLayout_5.setObjectName("verticalLayout_5")
        self.label_12 = QtWidgets.QLabel(self.about)
        self.label_12.setText("Main Developer")
        self.label_12.setObjectName("label_12")
        self.verticalLayout_5.addWidget(self.label_12)
        self.main_developer_text = QtWidgets.QTextBrowser(self.about)
        self.main_developer_text.setObjectName("main_developer_text")
        self.verticalLayout_5.addWidget(self.main_developer_text)
        self.horizontalLayout_7.addLayout(self.verticalLayout_5)
        self.verticalLayout_6 = QtWidgets.QVBoxLayout()
        self.verticalLayout_6.setObjectName("verticalLayout_6")
        self.label_13 = QtWidgets.QLabel(self.about)
        self.label_13.setText("All Contributors")
        self.label_13.setObjectName("label_13")
        self.verticalLayout_6.addWidget(self.label_13)
        self.all_contributor_text = QtWidgets.QTextBrowser(self.about)
        self.all_contributor_text.setObjectName("all_contributor_text")
        self.verticalLayout_6.addWidget(self.all_contributor_text)
        self.horizontalLayout_7.addLayout(self.verticalLayout_6)
        self.verticalLayout_7.addLayout(self.horizontalLayout_7)
        self.tabWidget.addTab(self.about, "About")
        self.verticalLayout_2.addWidget(self.tabWidget)
        self.horizontalLayout_2 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_2.setObjectName("horizontalLayout_2")
        spacerItem12 = QtWidgets.QSpacerItem(
            40, 20, QtWidgets.QSizePolicy.Expanding, QtWidgets.QSizePolicy.Minimum
        )
        self.horizontalLayout_2.addItem(spacerItem12)
        self.ok_btn = QtWidgets.QPushButton(Setting)
        self.ok_btn.setText("Save And Apply")
        self.ok_btn.setObjectName("ok_btn")
        self.horizontalLayout_2.addWidget(self.ok_btn)
        self.cancel_btn = QtWidgets.QPushButton(Setting)
        self.cancel_btn.setText("Cancel")
        self.cancel_btn.setObjectName("cancel_btn")
        self.horizontalLayout_2.addWidget(self.cancel_btn)
        self.verticalLayout_2.addLayout(self.horizontalLayout_2)

        self.retranslateUi(Setting)
        self.tabWidget.setCurrentIndex(0)
        QtCore.QMetaObject.connectSlotsByName(Setting)

    def retranslateUi(self, Setting):
        _translate = QtCore.QCoreApplication.translate
        self.pushButton_3.setText(_translate("Setting", "Github"))
