from ui.sessionSetting import Ui_SessionSetting


class SessionSettingUI(Ui_SessionSetting):
    def __init__(self, ourchat, dialog) -> None:
        self.ourchat = ourchat
        self.dialog = dialog

    def setupUi(self) -> None:
        super().setupUi(self.dialog)

        self.fillText()
        self.bind()

    def fillText(self):
        pass

    def bind(self):
        pass
