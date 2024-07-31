from PyQt5.QtWidgets import QDialog, QWidget
from PyQt5.QtGui import QCloseEvent


class AutoDestroyQDialog(QDialog):
    def __init__(self, ourchat, parent: QWidget | None = ...) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(parent)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeDialog(self)
        return super().closeEvent(a0)
