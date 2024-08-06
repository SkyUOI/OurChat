from ui.setting import Ui_Setting as Ui_Setting_NOLOGIC
from lib.const import log_level2str, str2log_level
from lib.OurChatUI import ImageLabel
from lib import OurChat
from PyQt6.QtWidgets import QMessageBox
import webbrowser


class Ui_Setting(Ui_Setting_NOLOGIC):
    def __init__(self, ourchat, widget):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget
        self.cache_config = OurChat.OurChatConfig()
        self.filling = False

    def setupUi(self):
        super().setupUi(self.widget)
        self.ok_btn.setEnabled(False)
        self.logo_label.deleteLater()
        self.logo_label = ImageLabel(self.widget)
        self.horizontalLayout_5.insertWidget(0, self.logo_label)
        self.fillText()
        self.bind()

    def fillText(self):
        self.filling = True
        self.cache_config.setConfig(self.ourchat.config)
        self.tabWidget.setTabText(0, self.ourchat.language["server"])
        self.tabWidget.setTabText(1, self.ourchat.language["general"])
        self.tabWidget.setTabText(2, self.ourchat.language["advanced"])
        self.tabWidget.setTabText(3, self.ourchat.language["about"])
        self.ip_label.setText(self.ourchat.language["ip"])
        self.port_label.setText(self.ourchat.language["port"])
        self.reconnection_attempt_label.setText(
            self.ourchat.language["reconnection_attempt"]
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

        self.ip_editor.setText(self.ourchat.config["server"]["ip"])
        self.port_editor.setValue(self.ourchat.config["server"]["port"])
        self.reconnection_attempt_editor.setValue(
            self.ourchat.config["server"]["reconnection_attempt"]
        )

        self.theme_combobox.clear()
        self.theme_combobox.addItems(self.uisystem.getThemes())
        self.theme_combobox.setCurrentText(self.ourchat.config["general"]["theme"])

        self.language_combobox.clear()
        self.language_combobox.addItems(self.ourchat.getLanguages())
        self.language_combobox.setCurrentText(
            self.ourchat.config["general"]["language"]
        )

        self.log_level_combobox.clear()
        self.log_level_combobox.addItems(
            ["DEBUG", "INFO", "WARNING", "CRITICAL", "ERROR"]
        )
        self.log_level_combobox.setCurrentText(
            log_level2str[self.ourchat.config["advanced"]["log_level"]]
        )

        self.log_saving_combobox.clear()
        self.log_saving_combobox.addItems(
            [
                "1",
                "3",
                "7",
                "15",
                "30",
                "60",
                "90",
                "180",
                "365",
                self.ourchat.language["always"],
            ]
        )
        log_saving_limit = self.ourchat.config["advanced"]["log_saving_limit"]
        if log_saving_limit == -1:
            log_saving_limit = self.ourchat.language["always"]
        else:
            log_saving_limit = str(log_saving_limit)
        self.log_saving_combobox.setCurrentText(log_saving_limit)

        self.logo_label.setImage("resources/images/logo.png")
        self.main_developer_text.setText(
            f'<a href="https://github.com/limuy2022" style="color:{self.uisystem.main_color}">Limuy</a><br><a href="https://github.com/senlinjun" style="color:{self.uisystem.main_color}">senlinjun</a>'
        )

        self.license_label.setText("License: GNU LICENSE 3.0")
        self.version_label.setText(
            f"Version: {self.ourchat.version_details['pkg_version']}({self.ourchat.version_details['branch']}+{self.ourchat.version_details['commit_hash']})"
        )
        self.filling = False

    def bind(self):
        self.ip_editor.textChanged.connect(self.valueChanged)
        self.port_editor.valueChanged.connect(self.valueChanged)
        self.reconnection_attempt_editor.valueChanged.connect(self.valueChanged)
        self.theme_combobox.currentTextChanged.connect(self.valueChanged)
        self.language_combobox.currentTextChanged.connect(self.valueChanged)
        self.log_level_combobox.currentTextChanged.connect(self.valueChanged)
        self.log_saving_combobox.currentTextChanged.connect(self.valueChanged)
        self.github_btn.clicked.connect(self.openGithub)

        self.ok_btn.clicked.connect(self.ok)
        self.cancel_btn.clicked.connect(self.widget.close)

    def valueChanged(self):
        if self.filling:
            return
        self.cache_config["server"]["ip"] = self.ip_editor.text()
        self.cache_config["server"]["port"] = self.port_editor.value()
        self.cache_config["server"]["reconnection_attempt"] = (
            self.reconnection_attempt_editor.value()
        )
        self.cache_config["general"]["theme"] = self.theme_combobox.currentText()
        self.cache_config["general"]["language"] = self.language_combobox.currentText()
        self.cache_config["advanced"]["log_level"] = str2log_level[
            self.log_level_combobox.currentText()
        ]
        log_saving_limit = self.log_saving_combobox.currentText()
        if log_saving_limit == self.ourchat.language["always"]:
            log_saving_limit = -1
        else:
            log_saving_limit = int(log_saving_limit)
        self.cache_config["advanced"]["log_saving_limit"] = log_saving_limit

        self.ok_btn.setEnabled(not self.cache_config.compareConfig(self.ourchat.config))

    def ok(self):
        flag = self.cache_config["server"] != self.ourchat.config["server"]
        self.ourchat.config.setConfig(self.cache_config)
        if flag:
            self.ourchat.restart(
                self.ourchat.language["server_information_is_modified"]
            )
            return
        self.ourchat.configUpdated()
        self.widget.close()

    def openGithub(self):
        response = webbrowser.open("https://github.com/SkyUOI/OurChat")
        if not response:
            QMessageBox.critical(
                self.widget,
                self.ourchat.language["error"],
                self.ourchat.language["open_github_fail"].format(
                    "https://github.com/SkyUOI/OurChat"
                ),
            )
