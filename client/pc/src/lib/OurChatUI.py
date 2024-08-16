from logging import getLogger
from typing import overload

from PyQt6.QtCore import QSize, Qt
from PyQt6.QtGui import QCloseEvent, QPixmap, QResizeEvent
from PyQt6.QtWidgets import (
    QDialog,
    QHBoxLayout,
    QLabel,
    QListWidgetItem,
    QSizePolicy,
    QSpacerItem,
    QTextBrowser,
    QVBoxLayout,
    QWidget,
)

logger = getLogger(__name__)


class OurChatDialog(QDialog):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeDialog(self)
        super().closeEvent(a0)


class OurChatWidget(QWidget):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeWidget(self)
        super().closeEvent(a0)


class ImageLabel(QLabel):
    @overload
    def setImage(self, path: str) -> None: ...

    @overload
    def setImage(self, data: bytes) -> None: ...

    def setImage(self, data) -> None:
        self.img = QPixmap()
        if isinstance(data, str):
            self.img.load(data)
        elif isinstance(data, bytes):
            self.img.loadFromData(data)
        else:
            logger.info("unknown data type")
            return
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
        super().resizeEvent(a0)


class SessionWidget(QWidget):
    def setSession(
        self, session_id: str, avatar: bytes, name: str, detail: str
    ) -> None:
        self.session_id = session_id
        main_layout = QHBoxLayout()
        self.setLayout(main_layout)

        img = ImageLabel(self)
        img.setImage(avatar)
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


class MessageWidget(QWidget):
    def setMessage(
        self,
        item: QListWidgetItem,
        avatar: bytes,
        name: str,
        message: str,
        me: bool = False,
    ) -> None:
        self.msg_id = None
        self.item = item
        self.avatar = avatar
        self.name = name
        self.message = message
        self.me = me

        self.layout = QHBoxLayout()
        self.avatar = ImageLabel(self)
        self.avatar.setMinimumHeight(40)
        self.avatar.setImage(avatar)
        self.avatar_layout = QVBoxLayout()
        self.avatar_layout.addWidget(self.avatar)
        self.avatar_layout.addSpacerItem(
            QSpacerItem(
                20, 40, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding
            )
        )
        self.detail_layout = QVBoxLayout()
        self.name_label = QLabel(self)
        self.name_label.setText(name)
        self.text_browser = QTextBrowser(self)
        self.text_browser.setAlignment(Qt.AlignmentFlag.AlignVCenter)
        self.text_browser.setVerticalScrollBarPolicy(
            Qt.ScrollBarPolicy.ScrollBarAlwaysOff
        )
        self.text_browser.setText(message)
        self.text_browser.document().adjustSize()
        self.text_browser.setMaximumWidth(
            self.text_browser.document().size().toSize().width() + 50
        )
        self.detail_layout.addWidget(self.name_label)
        self.detail_layout.addWidget(self.text_browser)
        if me:
            self.name_label.setAlignment(Qt.AlignmentFlag.AlignRight)
            self.layout.addSpacerItem(
                QSpacerItem(
                    40, 20, QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
                )
            )
            self.layout.addLayout(self.detail_layout)
            self.layout.addLayout(self.avatar_layout)
        else:
            self.name_label.setAlignment(Qt.AlignmentFlag.AlignLeft)
            self.layout.addLayout(self.avatar_layout)
            self.layout.addLayout(self.detail_layout)
            self.layout.addSpacerItem(
                QSpacerItem(
                    40, 20, QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
                )
            )
        self.setLayout(self.layout)
        item.setSizeHint(
            QSize(1, self.text_browser.document().size().toSize().height() + 70)
        )

    def resizeEvent(self, a0: QResizeEvent) -> None:
        self.item.setSizeHint(
            QSize(1, self.text_browser.document().size().toSize().height() + 70)
        )
        super().resizeEvent(a0)
