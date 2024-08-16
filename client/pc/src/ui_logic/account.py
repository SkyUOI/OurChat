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
        if self.ourchat.account.avatar_binary_data is not None:
            print(type(self.ourchat.account.avatar_binary_data))
            self.avatar_label.setImage(self.ourchat.account.avatar_binary_data)
        if "nickname" in self.ourchat.account.data:
            self.nickname_label.setText(self.ourchat.account.data["nickname"])

    def bind(self) -> None:
        pass
