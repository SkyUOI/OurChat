from io import BytesIO
from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    DEFAULT_IMAGE,
    HTTP_OK,
    RUN_NORMALLY,
    SERVER_ERROR,
    SERVER_UNDER_MAINTENANCE,
    SET_ACCOUNT_INFO_MSG,
    SET_ACCOUNT_INFO_RESPONSE_MSG,
    UNKNOWN_ERROR,
    UPLOAD_RESPONSE,
)
from lib.OurChatUI import ImageLabel
from PIL import Image
from PyQt6.QtWidgets import QFileDialog, QMessageBox
from ui.profile import Ui_Profile
from ui_logic.basicUI import BasicUI

logger = getLogger(__name__)


class ProfileUI(BasicUI, Ui_Profile):
    def __init__(self, ourchat, dialog) -> None:
        self.ourchat = ourchat
        self.dialog = dialog
        self.avatar_path = None

    def setupUi(self) -> None:
        super().setupUi(self.dialog)
        self.avatar_label.deleteLater()
        self.avatar_label = ImageLabel(self.dialog)
        self.verticalLayout.insertWidget(0, self.avatar_label)
        self.listen()
        self.fillText()
        self.bind()

    def listen(self) -> None:
        self.ourchat.listen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountInfoResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountAvatarResponse)
        self.ourchat.listen(SET_ACCOUNT_INFO_RESPONSE_MSG, self.setAccountInfoResponse)

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account != self.ourchat.account:
            return
        self.nickname_editor.setText(account.data["nickname"])

    def getAccountAvatarResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account != self.ourchat.account:
            return
        self.avatar_label.setImage(account.avatar_data)

    def fillText(self) -> None:
        self.label_2.setText(self.ourchat.language["nickname"])
        self.ok_btn.setText(self.ourchat.language["ok"])
        self.cancel_btn.setText(self.ourchat.language["cancel"])
        self.set_avatar_btn.setText(self.ourchat.language["set"])
        self.dialog.setWindowTitle(f"OurChat - {self.ourchat.language['profile']}")
        self.avatar_label.setImage(DEFAULT_IMAGE)
        account = self.ourchat.account
        if account.have_got_info:
            self.nickname_editor.setText(account.data["nickname"])
        if account.have_got_avatar:
            self.avatar_label.setImage(account.avatar_data)

    def bind(self) -> None:
        self.ok_btn.clicked.connect(self.ok)
        self.cancel_btn.clicked.connect(self.dialog.close)
        self.set_avatar_btn.clicked.connect(self.setAvatar)

    def ok(self) -> None:
        if self.avatar_path is not None:
            self.ourchat.listen(UPLOAD_RESPONSE, self.uploadResponse)
            self.ourchat.upload(self.avatar_data)
        else:
            self.updateInfo()

    def uploadResponse(self, data):
        self.ourchat.unListen(UPLOAD_RESPONSE, self.uploadResponse)
        if data["status"] == HTTP_OK:
            self.updateInfo(
                {
                    "avatar": f"http://{self.ourchat.config['server']['ip']}:{self.ourchat.config['server']['port']+1}/download",
                    "avatar_key": data["key"],
                }
            )
        else:
            logger.warning(f"upload avatar failed: CODE {data['status']}")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["upload_failed"].format(data["status"]),
            )
        return

    def updateInfo(self, data: dict = {}) -> None:
        if self.nickname_editor.text() != self.ourchat.account.data["nickname"]:
            data["nickname"] = self.nickname_editor.text()
        if data == {}:
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["account_profile_no_change"],
            )
            return
        self.ourchat.runThread(
            self.ourchat.conn.send,
            {
                "code": SET_ACCOUNT_INFO_MSG,
                "ocid": self.ourchat.account.ocid,
                "data": data,
            },
        )
        self.ourchat.runThread(self.ourchat.account.getInfo)

    def setAvatar(self) -> None:
        avatar_path = QFileDialog.getOpenFileName(
            self.dialog, filter="Image Files (*.png *.jpg *.jpeg *.gif)"
        )[0]
        if avatar_path == "":
            return
        self.avatar_path = avatar_path
        img = Image.open(self.avatar_path)
        bytes_io = BytesIO()
        img.resize((256, 256)).save(bytes_io, format="PNG")
        self.avatar_data = bytes_io.getvalue()
        self.avatar_label.setImage(bytes_io.getvalue())

    def setAccountInfoResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            self.dialog.close()
        elif data["status"] == SERVER_ERROR:
            logger.warning("set account info failed: server error")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("set account info failed: server under maintenance")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("set account info failed: unknown error")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )
