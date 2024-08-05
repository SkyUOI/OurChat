from PyQt6.QtWidgets import QDialog, QWidget, QLabel
from PyQt6.QtGui import QCloseEvent, QPixmap, QResizeEvent
from PyQt6.QtCore import Qt


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


class ImageLabel(QLabel):
    def setImage(self, path):
        self.img = QPixmap(path)
        size = self.size()
        size.setWidth(max(size.width(), 200) - 20)
        scaled_img = self.img.scaled(
            size, aspectRatioMode=Qt.AspectRatioMode.KeepAspectRatio
        )
        self.setPixmap(scaled_img)

    def resizeEvent(self, a0: QResizeEvent) -> None:
        size = self.size()
        size.setWidth(max(size.width(), 200) - 20)
        scaled_img = self.img.scaled(
            size, aspectRatioMode=Qt.AspectRatioMode.KeepAspectRatio
        )
        self.setPixmap(scaled_img)
        return super().resizeEvent(a0)
