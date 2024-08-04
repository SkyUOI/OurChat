from ui.setting import Ui_Setting as Ui_Setting_NOLOGIC


class Ui_Setting(Ui_Setting_NOLOGIC):
    def __init__(self, ourchat, widget):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget

    def setupUi(self):
        super().setupUi(self.widget)
        self.fillText()
        self.bind()

    def fillText(self):
        self.tabWidget.setTabText(0, self.ourchat.language["server"])
        self.tabWidget.setTabText(1, self.ourchat.language["general"])
        self.tabWidget.setTabText(2, self.ourchat.language["advanced"])
        self.tabWidget.setTabText(3, self.ourchat.language["about"])
        self.ip_label.setText(self.ourchat.language["ip"])
        self.port_label.setText(self.ourchat.language["port"])
        self.reconnection_attemp_label.setText(
            self.ourchat.language["reconnection_attemp"]
        )
        self.language_label.setText(self.ourchat.language["language"])
        self.theme_label.setText(self.ourchat.language["theme"])
        self.log_level_label.setText(self.ourchat.language["log_level"])
        self.log_saving_limit_label.setText(self.ourchat.language["log_saving_limit"])
        self.days_label.setText(self.ourchat.language["days"])
        self.main_developer_label.setText(self.ourchat.language["main_developer"])
        self.all_contributor_label.setText(self.ourchat.language["all_contributor"])
        self.ok_btn.setText(f'{self.ourchat.language["save&apply"]}')
        self.cancel_btn.setText(self.ourchat.language["cancel"])

    def bind(self):
        pass
