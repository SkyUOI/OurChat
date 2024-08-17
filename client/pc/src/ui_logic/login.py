import hashlib
from logging import getLogger
from typing import Union

from lib.const import (
    GENERATE_VERIFY_MSG,
    LOGIN_MSG,
    LOGIN_RESPONSE_MSG,
    REGISTER_MSG,
    REGISTER_RESPONSE_MSG,
    SERVER_STATUS_MSG,
    VERIFY_STATUS_MSG,
)
from lib.OurChatUI import OurChatWidget
from PyQt6.QtWidgets import QLineEdit, QMessageBox
from ui.login import Ui_Login
from ui_logic import setting

logger = getLogger(__name__)


class LoginUI(Ui_Login):
    def __init__(self, ourchat, widget: OurChatWidget) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(self) -> None:
        logger.info("setup Ui")
        super().setupUi(self.widget)
        self.join_btn.setEnabled(False)

        self.fillText()
        self.bind()

    def fillText(self) -> None:
        self.tabWidget.setTabText(0, self.ourchat.language["login"])
        self.tabWidget.setTabText(1, self.ourchat.language["register"])
        self.ocid_email_label.setText(
            f'{self.ourchat.language["ocid"]}/{self.ourchat.language["email"]}'
        )
        self.login_password_label.setText(self.ourchat.language["password"])
        self.login_show_checkbox.setText(self.ourchat.language["show_password"])
        self.email_label.setText(self.ourchat.language["email"])
        self.register_password_label.setText(self.ourchat.language["password"])
        self.register_show_checkbox.setText(self.ourchat.language["show_password"])
        self.setting_btn.setText(self.ourchat.language["setting"])
        self.connect_server_btn.setText(self.ourchat.language["connect_server"])
        self.join_btn.setText(self.ourchat.language["join"])
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['login']}")

    def bind(self) -> None:
        self.join_btn.clicked.connect(self.join)
        self.connect_server_btn.clicked.connect(self.connectToServer)
        self.login_show_checkbox.clicked.connect(self.showPassword)
        self.register_show_checkbox.clicked.connect(self.showPassword)
        self.setting_btn.clicked.connect(self.showSetting)

    def join(self) -> None:
        index = self.tabWidget.currentIndex()
        if index:  # register
            logger.info("begin to register")
            self.ourchat.listen(VERIFY_STATUS_MSG, self.verifyResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send, None, {"code": GENERATE_VERIFY_MSG}
            )
            QMessageBox.information(
                self.widget,
                self.ourchat.language["info"],
                self.ourchat.language["check_email"],
            )
        else:  # login
            logger.info("begin to login")
            sha256 = hashlib.sha256()
            sha256.update(self.login_password_editor.text().encode("ascii"))
            if "@" in self.login_account_editor.text():
                login_type = 0
            else:
                login_type = 1
            self.ourchat.listen(LOGIN_RESPONSE_MSG, self.loginResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send,
                None,
                {
                    "code": LOGIN_MSG,
                    "login_type": login_type,
                    "account": self.login_account_editor.text(),
                    "password": sha256.hexdigest(),
                },
            )

    def connectToServer(self) -> None:
        self.ourchat.runThread(self.ourchat.conn.connect, self.connectedServer)

    def connectedServer(self, result: Union[bool, str]) -> None:
        if result[0]:
            self.ourchat.runThread(self.ourchat.conn.recv)
            self.ourchat.listen(SERVER_STATUS_MSG, self.serverStatusResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send, None, {"code": SERVER_STATUS_MSG}
            )
        else:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["connect_server_fail"].format(result[1]),
            )

    def serverStatusResponse(self, result: dict) -> None:
        self.ourchat.unListen(SERVER_STATUS_MSG, self.serverStatusResponse)
        if result["status"] == 1:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["maintenance"],
            )
            logger.info("Maintenance in progress")
            return
        self.connect_server_btn.setEnabled(False)
        self.join_btn.setEnabled(True)

    def verifyResponse(self, result: dict) -> None:
        self.ourchat.unListen(VERIFY_STATUS_MSG, self.verifyResponse)
        if result["status"] == 0:
            self.ourchat.listen(REGISTER_RESPONSE_MSG, self.registerResponse)
            sha256 = hashlib.sha256()
            sha256.update(self.register_password_editor.text().encode("ascii"))
            logger.info("verify successfully,send register message")
            self.ourchat.runThread(
                self.ourchat.conn.send,
                None,
                {
                    "code": REGISTER_MSG,
                    "email": self.register_email_editor.text(),
                    "password": sha256.hexdigest(),
                },
            )
        elif result["status"] == 1:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["verify_error"],
            )

        elif result["status"] == 2:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["verify_timeout"],
            )

    def registerResponse(self, result: dict) -> None:
        if result["status"] == 0:
            logger.info("register success")
            QMessageBox.information(
                self.widget,
                self.ourchat.language["info"],
                self.ourchat.language["register_success"],
            )
            account = self.ourchat.getAccount(result["ocid"], True)
            self.ourchat.setAccount(account)
            self.uisystem.mainwindow.show()
            self.uisystem.ui_logic.show()
            self.widget.close()
        elif result["status"] == 1:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["server_error"],
            )
        elif result["status"] == 2:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["email_exist"],
            )
        self.ourchat.unListen(REGISTER_RESPONSE_MSG, self.registerResponse)

    def loginResponse(self, result: dict) -> None:
        if result["status"] == 0:
            logger.info("login success")
            account = self.ourchat.getAccount(result["ocid"], True)
            self.ourchat.setAccount(account)
            self.uisystem.ui_logic.show()
            self.uisystem.mainwindow.show()
            self.widget.close()
        elif result["status"] == 1:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["wrong_a/p"],
            )
        elif result["status"] == 2:
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["server_error"],
            )
        self.ourchat.unListen(LOGIN_RESPONSE_MSG, self.loginResponse)

    def showPassword(self, status: bool) -> None:
        logger.debug(f"show password: {status}")
        self.login_show_checkbox.setChecked(status)
        self.register_show_checkbox.setChecked(status)

        if status:
            echo_mode = QLineEdit.EchoMode.Normal
        else:
            echo_mode = QLineEdit.EchoMode.Password
        self.login_password_editor.setEchoMode(echo_mode)
        self.register_password_editor.setEchoMode(echo_mode)

    def showSetting(self) -> None:
        self.ourchat.uisystem.setWidget(setting.SettingUI, True).show()
