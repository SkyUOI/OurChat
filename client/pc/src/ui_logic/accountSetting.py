from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    SET_ACCOUNT_INFO_MSG,
)
from lib.OurChatUI import ImageLabel
from ui.accountSetting import Ui_AccountSetting


class AccountSettingUI(Ui_AccountSetting):
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

    def listen(self):
        self.ourchat.listen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountInfoResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountAvatarResponse)

    def getAccountInfoResponse(self, data) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account != self.ourchat.account:
            return
        self.nickname_editor.setText(account.data["nickname"])

    def getAccountAvatarResponse(self, data) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account != self.ourchat.account:
            return
        self.avatar_label.setImage(account.avatar_binary_data)
        self.avatar_url = account.data["avatar"]
        self.avatar_hash = account.data["avatar_hash"]

    def fillText(self):
        account = self.ourchat.account
        if account.have_got_info:
            self.nickname_editor.setText(account.data["nickname"])
        if account.have_got_avatar:
            self.avatar_label.setImage(account.avatar_binary_data)
            self.avatar_url = account.data["avatar"]
            self.avatar_hash = account.data["avatar_hash"]

    def bind(self):
        self.ok_btn.clicked.connect(self.ok)
        self.cancel_btn.clicked.connect(self.dialog.close)

    def ok(self):
        data = {}
        if self.nickname_editor.text() != self.ourchat.account.data["nickname"]:
            data["nickname"] = self.nickname_editor.text()
        if self.avatar_hash != self.ourchat.account.data["avatar_hash"]:
            data["avatar"] = self.avatar_url
            data["avatar_hash"] = self.avatar_hash
        if data == {}:
            return
        self.ourchat.send({"code": SET_ACCOUNT_INFO_MSG, "data": data})
