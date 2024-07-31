from ui.login import Ui_Login as Ui_Login_NOLOGIC
from lib.msg_code import SERVER_STATUS
from lib.msg_code import REGISTER
from lib.msg_code import REGISTER_RESPONSE
from lib.msg_code import LOGIN
from lib.msg_code import LOGIN_RESPONSE
from lib.msg_code import GENERATE_VERIFY
from lib.msg_code import VERIFY_STATUS
from PyQt5.QtWidgets import QMessageBox, QLineEdit
import hashlib


class Ui_Login(Ui_Login_NOLOGIC):
    def __init__(self, ourchat, dialog):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.dialog = dialog

    def setupUi(self):
        super().setupUi(self.dialog)
        self.needReconnect(None)
        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        self.join_btn.clicked.connect(self.join)
        self.connect_server_btn.clicked.connect(self.connectToServer)
        self.login_show_checkbox.clicked.connect(self.showPassword)
        self.register_show_checkbox.clicked.connect(self.showPassword)
        self.server_ip_editor.textChanged.connect(self.needReconnect)
        self.server_port_editor.textChanged.connect(self.needReconnect)

    def join(self):
        index = self.tabWidget.currentIndex()
        if index:  # register
            self.ourchat.listen(VERIFY_STATUS, self.verifyResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send, None, {"code": GENERATE_VERIFY}
            )
            QMessageBox.information(self.dialog, "Success", "Please check your email")
        else:  # login
            sha256 = hashlib.sha256()
            sha256.update(self.login_password_editor.text().encode("ascii"))
            if "@" in self.login_account_editor.text():
                login_type = 0
            else:
                login_type = 1
            self.ourchat.listen(LOGIN_RESPONSE, self.loginResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send,
                None,
                {
                    "code": LOGIN,
                    "login_type": login_type,
                    "account": self.login_account_editor.text(),
                    "password": sha256.hexdigest(),
                },
            )

    def connectToServer(self):
        self.ourchat.conn.setServer(
            self.server_ip_editor.text(), self.server_port_editor.text()
        )
        self.ourchat.runThread(self.ourchat.conn.connect, self.connectedServer)

    def connectedServer(self, result):
        if result[0]:
            self.ourchat.runThread(self.ourchat.conn.recv)
            self.ourchat.listen(SERVER_STATUS, self.serverStatusResponse)
            self.ourchat.runThread(
                self.ourchat.conn.send, None, {"code": SERVER_STATUS}
            )
        else:
            QMessageBox.warning(
                self.dialog, "Failed", f"Failed to connect to server:\n{result[1]}"
            )

    def serverStatusResponse(self, result):
        self.ourchat.unListen(SERVER_STATUS, self.serverStatusResponse)
        if result["status"] == 1:
            QMessageBox.warning(self.dialog, "Failed", "Maintenance in progress")
            return
        self.connect_server_btn.setEnabled(False)
        self.join_btn.setEnabled(True)

    def verifyResponse(self, result):
        self.ourchat.unListen(VERIFY_STATUS, self.verifyResponse)
        if result["status"] == 0:
            self.ourchat.listen(REGISTER_RESPONSE, self.registerResponse)
            sha256 = hashlib.sha256()
            sha256.update(self.register_password_editor.text().encode("ascii"))
            self.ourchat.runThread(
                self.ourchat.conn.send,
                None,
                {
                    "code": REGISTER,
                    "email": self.register_email_editor.text(),
                    "password": sha256.hexdigest(),
                },
            )
        elif result["status"] == 1:
            QMessageBox.warning(self.dialog, "ERROR", "Verify Error")

        elif result["status"] == 2:
            QMessageBox.warning(self.dialog, "ERROR", "Verify Timeout")

    def registerResponse(self, result):
        if result["status"] == 0:
            QMessageBox.information(self.dialog, "Success", "Register Success")
            self.uisystem.mainwindow.show()
            self.dialog.close()
        elif result["status"] == 1:
            QMessageBox.warning(self.dialog, "ERROR", "Server Error")
        elif result["status"] == 2:
            QMessageBox.warning(self.dialog, "ERROR", "Email Exist")
        self.ourchat.unListen(REGISTER_RESPONSE, self.registerResponse)

    def loginResponse(self, result):
        if result["status"] == 0:
            self.uisystem.mainwindow.show()
            self.dialog.close()
        elif result["status"] == 1:
            QMessageBox.warning(self.dialog, "ERROR", "Account/Password Error")
        elif result["status"] == 2:
            QMessageBox.warning(self.dialog, "ERROR", "Server Error")
        self.ourchat.unListen(LOGIN_RESPONSE, self.loginResponse)

    def showPassword(self, status):
        self.login_show_checkbox.setChecked(status)
        self.register_show_checkbox.setChecked(status)

        if status:
            echo_mode = QLineEdit.Normal
        else:
            echo_mode = QLineEdit.Password
        self.login_password_editor.setEchoMode(echo_mode)
        self.register_password_editor.setEchoMode(echo_mode)

    def needReconnect(self, a):
        self.join_btn.setEnabled(False)
        self.connect_server_btn.setEnabled(True)
