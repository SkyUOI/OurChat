from lib.const import ACCOUNT_FINISH_GET_AVATAR, ACCOUNT_FINISH_GET_INFO
from lib.OurChatUI import ImageLabel, OurChatWidget
from ui.account import Ui_Account


class AccountUI(Ui_Account):
    def __init__(self, ourchat, widget: OurChatWidget) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(self) -> None:
        super().setupUi(self.widget)
        self.avatar_label.deleteLater()
        self.avatar_label = ImageLabel(self.widget)
        self.avatar_label.setImage("resources/images/logo.png")  # default
        self.horizontalLayout.insertWidget(0, self.avatar_label)
        self.fillText()
        self.bind()

    def fillText(self) -> None:
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['account']}")
        self.ocid_label.setText(self.ourchat.account.ocid)
        self.nickname_label.setText("Nickname")
        self.avatar_label.setImage("resources/images/logo.png")
        self.logout_btn.setText(self.ourchat.language["logout"])
        self.unregister_btn.setText(self.ourchat.language["unregister"])
        if self.ourchat.account.have_got_info:
            self.nickname_label.setText(self.ourchat.account.data["nickname"])
        else:
            self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        if self.ourchat.account.have_got_avatar:
            self.avatar_label.setImage(self.ourchat.account.avatar_binary_data)
        else:
            self.ourchat.listen(
                ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse
            )

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.ourchat.unListen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
            self.nickname_label.setText(account.data["nickname"])

    def getAccountAvatarResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.ourchat.unListen(
                ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse
            )
            self.avatar_label.setImage(account.avatar_binary_data)

    def bind(self) -> None:
        self.logout_btn.clicked.connect(self.ourchat.restart)
