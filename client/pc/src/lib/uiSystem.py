from PyQt6.QtWidgets import QApplication, QMainWindow
from PyQt6.QtCore import QTimer, QDir
from lib.OurChatUI import OurChatDialog, OutChatWidget
from PyQt6.QtGui import QFontDatabase
from logging import getLogger
from ui_logic.login import Ui_Login
from ui_logic.main import Ui_Main
import os

logger = getLogger(__name__)


class UISystem:
    def __init__(self, ourchat, argv):
        logger.info("UISystem init")
        self.ourchat = ourchat
        self.app = QApplication(argv)
        self.mainwindow = QMainWindow()
        self.ui = None
        self.theme = None
        self.main_color = "#000000"
        self.dialogs = [
            # (dialog,ui_obj)
        ]
        self.widgets = [
            # (widget,ui_obj)
        ]
        self.tick_timer = QTimer()
        self.tick_timer.timeout.connect(self.ourchat.tick)
        self.tick_timer.start(10)
        QFontDatabase.addApplicationFont("resources/fonts/Roboto-Medium.ttf")
        QFontDatabase.addApplicationFont("resources/fonts/MiSans-Medium.ttf")

    def setUI(self, ui_class):
        logger.info(f"setUi to {ui_class.__qualname__}")
        self.ui = ui_class(self.ourchat)
        self.ui.setupUi()
        return self.mainwindow

    def exec(self):
        logger.info("start ui mainloop")
        self.app.exec()

    def setDialog(self, dialog_class, only=False):
        logger.info(f"new dialog {dialog_class.__qualname__}")
        new_dialog = OurChatDialog(self.ourchat)
        new_dialog_ui = dialog_class(self.ourchat, new_dialog)

        if only:
            remove_later = []
            for index in range(len(self.dialogs)):
                dialog, dialog_ui = self.dialogs[index]
                if type(dialog_ui) is type(new_dialog_ui):
                    dialog.destroy()
                    remove_later.append(index)

            for i in range(len(remove_later)):
                logger.info(
                    f"remove dialog {self.dialogs[remove_later[-1]][1].__class__.__qualname__}"
                )
                self.dialogs.pop(remove_later[-1])
                remove_later.pop(-1)

        new_dialog_ui.setupUi()
        self.dialogs.append((new_dialog, new_dialog_ui))
        logger.info(f"add dialog {dialog_class.__qualname__}")
        return new_dialog

    def setWidget(self, widget_class, only=False):
        logger.info(f"new widget {widget_class.__qualname__}")
        new_widget = OutChatWidget(self.ourchat)
        new_widget_ui = widget_class(self.ourchat, new_widget)

        if only:
            remove_later = []
            for index in range(len(self.widgets)):
                widget, widget_ui = self.widgets[index]
                if type(widget_ui) is type(new_widget_ui):
                    widget.destroy()
                    remove_later.append(index)

            for i in range(len(remove_later)):
                logger.info(
                    f"remove widget {self.widgets[remove_later[-1]][1].__class__.__qualname__}"
                )
                self.widgets.pop(remove_later[-1])
                remove_later.pop(-1)

        new_widget_ui.setupUi()
        self.widgets.append((new_widget, new_widget_ui))
        logger.info(f"add widget {widget_class.__qualname__}")
        return new_widget

    def removeDialog(self, rm_dialog):
        for i in range(len(self.dialogs)):
            dialog, dialog_ui = self.dialogs[i]
            if dialog == rm_dialog:
                logger.info(f"remove dialog {dialog_ui.__class__.__qualname__}")
                self.dialogs.pop(i)
                break

    def removeWidget(self, rm_widget):
        for i in range(len(self.widgets)):
            widget, widget_ui = self.widgets[i]
            if widget == rm_widget:
                logger.info(f"remove widget {widget_ui.__class__.__qualname__}")
                self.widgets.pop(i)
                break

    def setTheme(self, theme, font_family):
        if theme is None:
            return
        QDir.setSearchPaths("icon", [f"theme/{theme}/resources"])
        with open(f"theme/{theme}/{theme}.qss", "r") as f:
            qss = f.read()
        qss.replace("{FONT_FAMILY}", font_family)
        self.main_color = qss[
            qss.index(
                "COLOR: ",
            )
            + len("COLOR: ") :
        ].split(";")[0]
        self.app.setStyleSheet(qss)

    def getThemes(self):
        return os.listdir("theme")

    def run(self):
        self.setUI(Ui_Main)
        widget = self.setWidget(Ui_Login, True)
        widget.show()

    def configUpdated(self):
        self.setTheme(
            self.ourchat.config["general"]["theme"],
            self.ourchat.language["FONT_FAMILY"],
        )
        for widget, widget_ui in self.widgets:
            widget_ui.fillText()
        for dialog, dialog_ui in self.dialogs:
            dialog_ui.fillText()
        if self.ui is not None:
            self.ui.fillText()
