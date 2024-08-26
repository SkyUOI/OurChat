import hashlib
import re
from logging import getLogger
from typing import Union

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    RUN_NORMALLY,
    SERVER_ERROR,
    SERVER_UNDER_MAINTENANCE,
    SET_ACCOUNT_INFO_MSG,
    SET_ACCOUNT_INFO_RESPONSE_MSG,
    UNKNOWN_ERROR,
)
from lib.OurChatUI import ImageLabel
from PyQt6.QtWidgets import QInputDialog, QMessageBox
from ui.profile import Ui_Profile

logger = getLogger(__name__)


class ProfileUI(Ui_Profile):
    def __init__(self, ourchat, dialog) -> None:
        self.ourchat = ourchat
        self.dialog = dialog

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
        self.avatar_url = account.data["avatar"]
        self.avatar_hash = account.data["avatar_hash"]

    def fillText(self) -> None:
        self.label_2.setText(self.ourchat.language["nickname"])
        self.ok_btn.setText(self.ourchat.language["ok"])
        self.cancel_btn.setText(self.ourchat.language["cancel"])
        self.set_avatar_btn.setText(self.ourchat.language["set"])
        self.dialog.setWindowTitle(f"OurChat - {self.ourchat.language['profile']}")
        account = self.ourchat.account
        if account.have_got_info:
            self.nickname_editor.setText(account.data["nickname"])
        if account.have_got_avatar:
            self.avatar_label.setImage(account.avatar_data)
            self.avatar_url = account.data["avatar"]
            self.avatar_hash = account.data["avatar_hash"]

    def bind(self) -> None:
        self.ok_btn.clicked.connect(self.ok)
        self.cancel_btn.clicked.connect(self.dialog.close)
        self.set_avatar_btn.clicked.connect(self.setAvatar)

    def ok(self) -> None:
        data = {}
        if self.nickname_editor.text() != self.ourchat.account.data["nickname"]:
            data["nickname"] = self.nickname_editor.text()
        if self.avatar_hash != self.ourchat.account.data["avatar_hash"]:
            data["avatar"] = self.avatar_url
            data["avatar_hash"] = self.avatar_hash
        if data == {}:
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["account_profile_no_change"],
            )
            return
        self.ourchat.conn.send(
            {
                "code": SET_ACCOUNT_INFO_MSG,
                "ocid": self.ourchat.account.ocid,
                "data": data,
            }
        )
        self.ourchat.runThread(self.ourchat.account.getInfo)

    def setAvatar(self) -> None:
        url = QInputDialog.getText(self.dialog, "set avatar", "avatar url: ")
        if url[1]:
            result = re.match(
                "^(https?:\/\/)?([\w-]+(?:\.[\w-]+)+)(:\d+)?(\/\S*)?$", url[0]
            )
            if result is None:
                QMessageBox.warning(
                    self.dialog,
                    self.ourchat.language["warning"],
                    self.ourchat.language["invalid_url"],
                )
                return
            self.ok_btn.setEnabled(False)
            self.avatar_url = url[0]
            self.ourchat.runThread(
                self.ourchat.download, self.downloadAvatarResponse, url[0]
            )

    def downloadAvatarResponse(self, avatar_data: Union[bytes, None]) -> None:
        if avatar_data is None:
            self.ok_btn.setEnabled(True)
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["avatar_download_failed"],
            )
            return
        hash = hashlib.sha256()
        hash.update(avatar_data)
        self.ourchat.cache.setImage(hash.hexdigest(), avatar_data)
        self.avatar_hash = hash.hexdigest()
        self.avatar_label.setImage(avatar_data)
        self.ok_btn.setEnabled(True)

    def setAccountInfoResponse(self, data: dict) -> None:
        if data["status_code"] == RUN_NORMALLY:
            self.dialog.close()
        elif data["status_code"] == SERVER_ERROR:
            logger.warning("set account info failed: server error")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status_code"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("set account info failed: server under maintenance")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status_code"] == UNKNOWN_ERROR:
            logger.warning("set account info failed: unknown error")
            QMessageBox.warning(
                self.dialog,
                self.ourchat.language["warning"],
                self.ourchat.language["set_account_info_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )
