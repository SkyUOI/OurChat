import os
from logging import getLogger

from lib.OurChatUI import OurChatDialog, OurChatMainWindow, OurChatWidget
from PyQt6.QtCore import QDir, QTimer
from PyQt6.QtGui import QFontDatabase, QIcon
from PyQt6.QtWidgets import QApplication
from ui_logic.login import LoginUI
from ui_logic.main import MainUI

logger = getLogger(__name__)


class UISystem:
    def __init__(self, ourchat, argv):
        logger.info("UISystem init")
        self.ourchat = ourchat
        self.app = QApplication(argv)
        self.mainwindow = None
        self.ui_logic = None
        self.theme = None
        self.main_color = "#000000"
        self.dialogs = [
            # (dialog,ui_logic)
        ]
        self.widgets = [
            # (widget,ui_logic)
        ]
        self.tick_timer = QTimer()
        self.tick_timer.timeout.connect(self.ourchat.tick)
        QFontDatabase.addApplicationFont("resources/fonts/Roboto-Medium.ttf")
        QFontDatabase.addApplicationFont("resources/fonts/MiSans-Medium.ttf")

    def setUI(self, ui_class):
        logger.info(f"setUi to {ui_class.__qualname__}")
        self.ui_logic = ui_class(self.ourchat)
        self.ui_logic.setupUi()
        return self.mainwindow

    def exec(self):
        logger.info("start ui mainloop")
        self.app.exec()

    def setDialog(self, dialog_class, only=False):
        logger.info(f"new dialog {dialog_class.__qualname__}")
        new_dialog = OurChatDialog(self.ourchat)
        new_ui_logic = dialog_class(self.ourchat, new_dialog)

        if only:
            remove_later = []
            for index in range(len(self.dialogs)):
                dialog, ui_logic = self.dialogs[index]
                if type(ui_logic) is type(new_ui_logic):
                    dialog.destroy()
                    remove_later.append(index)

            for i in range(len(remove_later)):
                logger.info(
                    f"remove dialog {self.dialogs[remove_later[-1]][1].__class__.__qualname__}"
                )
                self.dialogs.pop(remove_later[-1])
                remove_later.pop(-1)

        new_ui_logic.setupUi()
        self.dialogs.append((new_dialog, new_ui_logic))
        logger.info(f"add dialog {dialog_class.__qualname__}")
        return new_dialog

    def setWidget(self, widget_class, only=False):
        logger.info(f"new widget {widget_class.__qualname__}")
        new_widget = OurChatWidget(self.ourchat)
        new_ui_logic = widget_class(self.ourchat, new_widget)

        if only:
            remove_later = []
            for index in range(len(self.widgets)):
                widget, ui_logic = self.widgets[index]
                if type(ui_logic) is type(new_ui_logic):
                    widget.destroy()
                    remove_later.append(index)

            for i in range(len(remove_later)):
                logger.info(
                    f"remove widget {self.widgets[remove_later[-1]][1].__class__.__qualname__}"
                )
                self.widgets.pop(remove_later[-1])
                remove_later.pop(-1)

        new_ui_logic.setupUi()
        self.widgets.append((new_widget, new_ui_logic))
        logger.info(f"add widget {widget_class.__qualname__}")
        return new_widget

    def removeDialog(self, rm_dialog):
        for i in range(len(self.dialogs)):
            dialog, ui_logic = self.dialogs[i]
            if dialog == rm_dialog:
                logger.info(f"remove dialog {ui_logic.__class__.__qualname__}")
                self.dialogs.pop(i)
                break

    def removeWidget(self, rm_widget):
        for i in range(len(self.widgets)):
            widget, ui_logic = self.widgets[i]
            if widget == rm_widget:
                logger.info(f"remove widget {ui_logic.__class__.__qualname__}")
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
        self.mainwindow = OurChatMainWindow(self.ourchat)
        self.setUI(MainUI)
        self.app.setWindowIcon(QIcon("resources/images/logo.ico"))
        widget = self.setWidget(LoginUI, True)
        widget.show()
        self.tick_timer.start(1)

    def configUpdated(self):
        self.setTheme(
            self.ourchat.config["general"]["theme"],
            self.ourchat.language["FONT_FAMILY"],
        )
        for widget, ui_logic in self.widgets:
            ui_logic.fillText()
        for dialog, ui_logic in self.dialogs:
            ui_logic.fillText()
        if self.ui_logic is not None:
            self.ui_logic.fillText()

    def close(self):
        self.mainwindow.close()
        self.app.closeAllWindows()
        self.tick_timer.stop()

    def widgetKeyPressEvent(self, widget, event):
        for widget, ui_logic in self.widgets:
            if widget == widget:
                ui_logic.keyPressEvent(event)

    def dialogKeyPressEvent(self, dialog, event):
        for dialog, ui_logic in self.dialogs:
            if dialog == dialog:
                ui_logic.keyPressEvent(event)

    def mainWindowKeyPressEvent(self, event):
        self.ui_logic.keyPressEvent(event)
