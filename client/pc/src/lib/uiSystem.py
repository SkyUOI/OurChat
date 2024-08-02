from PyQt5.QtWidgets import QApplication, QMainWindow
from PyQt5.QtCore import Qt, QTimer, QDir
from lib.OurChatUI import AutoDestroyQDialog, AutoDestroyQWidget
from PyQt5.QtGui import QFontDatabase
from logging import getLogger

logger = getLogger(__name__)


class UISystem:
    def __init__(self, ourchat, argv):
        logger.info("UISystem init")
        self.ourchat = ourchat
        self.app = self.createApp(argv)
        self.mainwindow = QMainWindow()
        self.ui = None
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
        self.setTheme("dark_amber")

    def createApp(self, argv):
        logger.info("createApp")
        QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
        QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)
        QApplication.setHighDpiScaleFactorRoundingPolicy(
            Qt.HighDpiScaleFactorRoundingPolicy.Round
        )
        return QApplication(argv)

    def setUI(self, ui_class):
        logger.info(f"setUi to {ui_class.__qualname__}")
        self.ui = ui_class(self.ourchat)
        self.ui.setupUi()
        return self.mainwindow

    def exec(self):
        logger.info("start ui mainloop")
        self.app.exec_()

    def setDialog(self, dialog_class, only=False):
        logger.info(f"new dialog {dialog_class.__qualname__}")
        new_dialog = AutoDestroyQDialog(self.ourchat, self.mainwindow)
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
        new_widget = AutoDestroyQWidget(self.ourchat)
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

    def setTheme(self, theme):
        QDir.setSearchPaths("icon", [f"theme/{theme}/resources"])
        with open(f"theme/{theme}/{theme}.qss", "r") as f:
            qss = f.read()
        self.app.setStyleSheet(qss)
