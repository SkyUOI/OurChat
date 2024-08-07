from PyQt6.QtWidgets import (
    QDialog,
    QWidget,
    QLabel,
    QVBoxLayout,
    QHBoxLayout,
    QSpacerItem,
    QSizePolicy,
)
from PyQt6.QtGui import QCloseEvent, QPixmap, QResizeEvent
from PyQt6.QtCore import Qt


class OurChatDialog(QDialog):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeDialog(self)
        return super().closeEvent(a0)


class OutChatWidget(QWidget):
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


class SessionWidget(QWidget):
    def setSession(self, avatar_path, name, detail):
        main_layout = QHBoxLayout()
        self.setLayout(main_layout)

        img = ImageLabel(self)
        img.setImage(avatar_path)
        main_layout.addWidget(img)

        info_layout = QVBoxLayout()
        name_label = QLabel(self)
        name_label.setText(name)
        detail_label = QLabel(self)
        detail_label.setText(detail)
        info_layout.addWidget(name_label)
        info_layout.addWidget(detail_label)
        main_layout.addLayout(info_layout)

        spacer_item = QSpacerItem(
            40, 20, QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
        )
        main_layout.addSpacerItem(spacer_item)
