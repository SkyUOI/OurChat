from PyQt5.QtWidgets import QApplication, QMainWindow
from PyQt5.QtCore import Qt, QTimer
from lib.autoDestroyQDialog import AutoDestroyQDialog
from qt_material import apply_stylesheet


class UISystem:
    def __init__(self, ourchat, argv):
        self.ourchat = ourchat
        self.app = self.createApp(argv)
        self.mainwindow = QMainWindow()
        self.ui = None
        self.dialogs = [
            # (dialog,ui_obj)
        ]
        self.tick_timer = QTimer()
        self.tick_timer.timeout.connect(self.ourchat.tick)
        self.tick_timer.start(10)
        self.theme = "dark_amber.xml"

        apply_stylesheet(self.app, self.theme)

    def createApp(self, argv):
        QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
        QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)
        QApplication.setHighDpiScaleFactorRoundingPolicy(
            Qt.HighDpiScaleFactorRoundingPolicy.Round
        )
        return QApplication(argv)

    def setUI(self, ui_class):
        self.ui = ui_class(self.ourchat)
        self.ui.setupUi()
        return self.mainwindow

    def exec(self):
        self.app.exec_()

    def setDialog(self, dialog_class, only=False):
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
                self.dialogs.pop(remove_later[-1])
                remove_later.pop(-1)

        new_dialog_ui.setupUi()
        self.dialogs.append((new_dialog, new_dialog_ui))
        return new_dialog

    def removeDialog(self, rm_dialog):
        for i in range(len(self.dialogs)):
            dialog, dialog_ui = self.dialogs[i]
            if dialog == rm_dialog:
                self.dialogs.pop(i)
                break
