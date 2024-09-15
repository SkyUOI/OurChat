from io import BytesIO
from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    ACCOUNT_LIMIT,
    DEFAULT_IMAGE,
    HTTP_OK,
    NEW_SESSION_MSG,
    NEW_SESSION_RESPONSE_MSG,
    RUN_NORMALLY,
    SERVER_ERROR,
    SERVER_UNDER_MAINTENANCE,
    UNKNOWN_ERROR,
    UPLOAD_RESPONSE,
)
from lib.OurChatUI import AccountListItemWidget, ImageLabel
from PIL import Image
from PyQt6.QtWidgets import QFileDialog, QListWidgetItem, QMessageBox
from ui.sessionSetting import Ui_SessionSetting
from ui_logic.basicUI import BasicUI

logger = getLogger(__name__)


class SessionSettingUI(BasicUI, Ui_SessionSetting):
    def __init__(self, ourchat, dialog, session_id=None) -> None:
        self.ourchat = ourchat
        self.dialog = dialog
        self.session_id = session_id
        self.account_widgets = {}
        self.avatar_path = None

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
        self.dialog.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")
        self.session_name_editor.setPlaceholderText(
            self.ourchat.language["default_session_name"]
        )
        self.avatar_label.setImage(DEFAULT_IMAGE)
        for friend_ocid in self.ourchat.account.friends:
            friend_account = self.ourchat.getAccount(friend_ocid)
            self.addAccount(friend_account, False)

    def addAccount(self, account, checked: bool = False) -> None:
        avatar = DEFAULT_IMAGE
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
        if self.avatar_path is not None:
            self.ourchat.listen(UPLOAD_RESPONSE, self.uploadResponse)
            with open(self.avatar_path, "rb") as f:
                self.ourchat.upload(f.read())
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

    def updateInfo(self, data: dict = {}) -> None:
        members = [
            self.ourchat.account.ocid,
        ]
        for ocid in self.account_widgets:
            if self.account_widgets[ocid].checkbox.isChecked():
                members.append(ocid)
        self.ourchat.listen(NEW_SESSION_RESPONSE_MSG, self.newSessionResponse)
        data["code"] = NEW_SESSION_MSG
        data["members"] = members
        data["owner"] = self.ourchat.account.ocid
        if self.session_name_editor.text() != "":
            data["name"] = self.session_name_editor.text()
        self.ourchat.runThread(self.ourchat.conn.send, data)

    def newSessionResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            self.ourchat.unListen(NEW_SESSION_RESPONSE_MSG, self.newSessionResponse)
            session_id = data["session_id"]
            self.ourchat.getSession(session_id)
            self.ourchat.account.sessions.append(session_id)
            QMessageBox.information(
                self.dialog,
                self.ourchat.language["success"],
                self.ourchat.language["create_session_success"],
            )
            self.ourchat.account.getInfo()
            self.dialog.close()
        elif data["status"] == SERVER_ERROR:
            logger.warning("create session failed: server error")
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["warning"],
                self.ourchat.language["create_session_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("create session failed: server under maintenance")
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["warning"],
                self.ourchat.language["create_session_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == ACCOUNT_LIMIT:
            logger.warning("create session failed: reach sessions limit")
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["warning"],
                self.ourchat.language["create_session_failed"].format(
                    self.ourchat.language["reach_sessions_limit"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("create session failed: unknown error")
            QMessageBox.warning(
                self.widget,
                self.ourchat.language["warning"],
                self.ourchat.language["create_session_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )

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
