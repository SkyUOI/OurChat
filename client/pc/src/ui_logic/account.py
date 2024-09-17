from lib.const import ACCOUNT_FINISH_GET_AVATAR, ACCOUNT_FINISH_GET_INFO, DEFAULT_IMAGE
from lib.OurChatUI import ImageLabel, OurChatWidget
from ui.account import Ui_Account
from ui_logic.basicUI import BasicUI
from ui_logic.profile import ProfileUI


class AccountUI(BasicUI, Ui_Account):
    def __init__(self, ourchat, widget: OurChatWidget) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(self) -> None:
        super().setupUi(self.widget)
        self.avatar_label.deleteLater()
        self.avatar_label = ImageLabel(self.widget)
        self.horizontalLayout.insertWidget(1, self.avatar_label)
        self.listen()
        self.fillText()
        self.bind()

    def fillText(self) -> None:
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['account']}")
        self.ocid_label.setText(self.ourchat.account.ocid)
        self.nickname_label.setText("Nickname")
        self.avatar_label.setImage(DEFAULT_IMAGE)
        self.logout_btn.setText(self.ourchat.language["logout"])
        self.unregister_btn.setText(self.ourchat.language["unregister"])
        self.profile_btn.setText(self.ourchat.language["profile"])
        if self.ourchat.account.have_got_info:
            self.nickname_label.setText(self.ourchat.account.data["nickname"])
        if self.ourchat.account.have_got_avatar:
            self.avatar_label.setImage(self.ourchat.account.avatar_data)

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.nickname_label.setText(account.data["nickname"])

    def getAccountAvatarResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.avatar_label.setImage(account.avatar_data)

    def bind(self) -> None:
        self.logout_btn.clicked.connect(self.logout)
        self.profile_btn.clicked.connect(self.profile)

    def profile(self) -> None:
        dialog = self.uisystem.setDialog(ProfileUI, True)
        dialog.show()

    def logout(self) -> None:
        self.ourchat.restart()

    def listen(self):
        self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse)

    def close(self):
        self.ourchat.unListen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        self.ourchat.unListen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse)
