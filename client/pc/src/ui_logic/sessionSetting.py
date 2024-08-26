import hashlib
import re
from typing import Union

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    NEW_SESSION_MSG,
    NEW_SESSION_RESPONSE_MSG,
)
from lib.OurChatUI import AccountListItemWidget, ImageLabel
from PyQt6.QtWidgets import QInputDialog, QListWidgetItem, QMessageBox
from ui.sessionSetting import Ui_SessionSetting


class SessionSettingUI(Ui_SessionSetting):
    def __init__(self, ourchat, dialog, session_id=None) -> None:
        self.ourchat = ourchat
        self.dialog = dialog
        self.session_id = session_id
        self.account_widgets = {}
        self.avatar_url = None
        self.avatar_hash = None

    def setupUi(self) -> None:
        super().setupUi(self.dialog)
        self.avatar_label.deleteLater()
        self.avatar_label = ImageLabel(self.dialog)
        self.verticalLayout.insertWidget(0, self.avatar_label)

        self.member_list.verticalScrollBar().setSingleStep(10)

        self.listen()
        self.fillText()
        self.bind()

    def fillText(self):
        self.avatar_label.setImage("resources/images/logo.png")
        for friend_ocid in self.ourchat.account.friends:
            friend_account = self.ourchat.account.friends[friend_ocid]
            self.addAccount(friend_account, False)

    def addAccount(self, account, checked: bool = False) -> None:
        avatar = "resources/images/logo.png"
        nickname = "nickname"
        if account.have_got_info:
            nickname = account.data["nickname"]
        if account.have_got_avatar:
            avatar = account.avatar_data
        item = QListWidgetItem(self.member_list)
        widget = AccountListItemWidget(self.member_list)
        widget.setAccount(item, avatar, nickname, checked)
        self.member_list.addItem(item)
        self.member_list.setItemWidget(item, widget)
        self.account_widgets[account.ocid] = widget

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if data["ocid"] in self.account_widgets:
            self.account_widgets[data["ocid"]].setNickname(account.data["name"])

    def getAccountAvatarResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if data["ocid"] in self.account_widgets:
            self.account_widgets[data["ocid"]].setNickname(account.avatar_data)

    def bind(self):
        self.ok_btn.clicked.connect(self.ok)
        self.cancel_btn.clicked.connect(self.dialog.close)
        self.set_avatar_btn.clicked.connect(self.setAvatar)

    def listen(self):
        self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse)

    def ok(self) -> None:
        members = [
            self.ourchat.account.ocid,
        ]
        for ocid in self.account_widgets:
            if self.account_widgets[ocid].checkbox.isChecked():
                members.append(ocid)
        self.ourchat.listen(NEW_SESSION_RESPONSE_MSG, self.newSessionResponse)
        data = {
            "code": NEW_SESSION_MSG,
            "members": members,
        }
        if self.avatar_url is not None:
            data["avatar"] = self.avatar_url
            data["avatar_hash"] = hashlib.md5(
                self.avatar_url.encode("utf-8")
            ).hexdigest()
        if self.session_name_editor.text() != "":
            data["name"] = self.session_name_editor.text()
        self.ourchat.conn.send(data)
        self.dialog.close()

    def newSessionResponse(self, data: dict) -> None:
        self.ourchat.unListen(NEW_SESSION_RESPONSE_MSG, self.newSessionResponse)
        session_id = data["session_id"]
        self.ourchat.account.sessions[session_id] = self.ourchat.getSession(session_id)
        QMessageBox.information(
            self.dialog,
            self.ourchat.language["success"],
            self.ourchat.language["create_session_success"],
        )
        self.dialog.close()

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
            self.avatar_url = None
            self.avatar_hash = None
            self.avatar_label.setImage("resources/images/logo.png")
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
