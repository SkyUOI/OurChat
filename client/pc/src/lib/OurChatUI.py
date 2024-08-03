from PyQt6.QtWidgets import QDialog, QWidget
from PyQt6.QtGui import QCloseEvent


class AutoDestroyQDialog(QDialog):
    def __init__(self, ourchat, parent: QWidget | None = ...) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(parent)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeDialog(self)
        return super().closeEvent(a0)


class AutoDestroyQWidget(QWidget):
    def __init__(self, ourchat):
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeWidget(self)
        return super().closeEvent(a0)
