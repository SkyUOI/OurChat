from logging import getLogger
from typing import Any, overload

from lib.const import DEFAULT_IMAGE
from PyQt6.QtCore import QSize, Qt
from PyQt6.QtGui import QCloseEvent, QKeyEvent, QPixmap, QResizeEvent
from PyQt6.QtWidgets import (
    QCheckBox,
    QDialog,
    QHBoxLayout,
    QLabel,
    QListWidgetItem,
    QMainWindow,
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

    def keyPressEvent(self, a0: QKeyEvent) -> None:
        self.uisystem.dialogKeyPressEvent(self, a0)
        return super().keyPressEvent(a0)


class OurChatWidget(QWidget):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def closeEvent(self, a0: QCloseEvent) -> None:
        self.uisystem.removeWidget(self)
        super().closeEvent(a0)

    def keyPressEvent(self, a0: QKeyEvent) -> None:
        self.uisystem.widgetKeyPressEvent(self, a0)
        return super().keyPressEvent(a0)


class ImageLabel(QLabel):
    def __init__(self, parent):
        super().__init__(parent)
        self.setImage(DEFAULT_IMAGE)

    @overload
    def setImage(self, path: str) -> None: ...

    @overload
    def setImage(self, data: bytes) -> None: ...

    @overload
    def setAvatar(self, data: None) -> None: ...

    def setImage(self, data) -> None:
        self.img = QPixmap()
        if isinstance(data, str):
            self.img.load(data)
        elif isinstance(data, bytes):
            self.img.loadFromData(data)
        elif data is None:
            self.img.load(DEFAULT_IMAGE)
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


class SessionListItemWidget(QWidget):
    @overload
    def setSession(
        self, session_id: str, avatar: bytes, name: str, detail: str
    ) -> None: ...
    @overload
    def setSession(
        self, session_id: str, avatar: str, name: str, detail: str
    ) -> None: ...
    def setSession(self, session_id: str, avatar: Any, name: str, detail: str) -> None:
        self.session_id = session_id
        self.main_layout = QHBoxLayout()
        self.setLayout(self.main_layout)

        self.img = ImageLabel(self)
        self.img.setImage(avatar)
        self.main_layout.addWidget(self.img)

        self.info_layout = QVBoxLayout()
        self.name_label = QLabel(self)
        self.name_label.setText(name)
        self.detail_label = QLabel(self)
        self.detail_label.setText(detail)
        self.info_layout.addWidget(self.name_label)
        self.info_layout.addWidget(self.detail_label)
        self.main_layout.addLayout(self.info_layout)

        spacer_item = QSpacerItem(
            40, 20, QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
        )
        self.main_layout.addSpacerItem(spacer_item)

    def setName(self, name: str) -> None:
        self.name_label.setText(name)

    @overload
    def setAvatar(self, path: str) -> None: ...

    @overload
    def setAvatar(self, data: bytes) -> None: ...

    def setAvatar(self, data) -> None:
        self.img.setImage(data)

    def setDetail(self, detail: str) -> None:
        self.detail_label.setText(detail)


class MessageListItemWidget(QWidget):
    @overload
    def setMessage(
        self,
        item: QListWidgetItem,
        avatar: bytes,
        name: str,
        message: str,
        me: bool = False,
    ) -> None: ...
    @overload
    def setMessage(
        self,
        item: QListWidgetItem,
        avatar: str,
        name: str,
        message: str,
        me: bool = False,
    ) -> None: ...
    def setMessage(
        self,
        item: QListWidgetItem,
        avatar: Any,
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
        self.text_browser.setPlainText(message)
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

    @overload
    def setAvatar(self, path: str) -> None: ...

    @overload
    def setAvatar(self, data: bytes) -> None: ...

    @overload
    def setAvatar(self, data: None) -> None: ...

    def setAvatar(self, data) -> None:
        self.avatar.setImage(data)

    def setName(self, name: str):
        self.name_label.setText(name)

    def resizeEvent(self, a0: QResizeEvent) -> None:
        self.item.setSizeHint(
            QSize(1, self.text_browser.document().size().toSize().height() + 70)
        )
        super().resizeEvent(a0)


class AccountListItemWidget(QWidget):
    def setAccount(self, item: QListWidgetItem, avatar, nickname, checked=None):
        self.layout = QHBoxLayout(self)
        self.avatar_label = ImageLabel(self)
        self.avatar_label.setImage(avatar)
        self.layout.addWidget(self.avatar_label)
        self.nickname_label = QLabel(self)
        self.nickname_label.setText(nickname)
        self.layout.addWidget(self.nickname_label)
        self.layout.addSpacerItem(
            QSpacerItem(
                40, 20, QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Minimum
            )
        )
        if checked is not None:
            self.checkbox = QCheckBox(self)
            self.checkbox.setChecked(checked)
            self.layout.addWidget(self.checkbox)
        self.setLayout(self.layout)
        item.setSizeHint(QSize(60, 60))

    def setNickname(self, nickname: str):
        self.nickname_label.setText(nickname)

    @overload
    def setAvatar(self, avatar: bytes) -> None: ...

    @overload
    def setAvatar(self, path: str) -> None: ...

    @overload
    def setAvatar(self, data: None) -> None: ...

    def setAvatar(self, avatar):
        self.avatar_label.setImage(avatar)


class OurChatMainWindow(QMainWindow):
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        super().__init__(None)

    def keyPressEvent(self, a0: QKeyEvent) -> None:
        self.uisystem.mainWindowKeyPressEvent(a0)
        return super().keyPressEvent(a0)


class OurChatEditor(QTextBrowser):
    def __init__(self, parent: QWidget | None = ...) -> None:
        self.pressing_key = []
        self.hotkeys = []
        super().__init__(parent)

    def keyPressEvent(self, e: QKeyEvent) -> None:
        self.pressing_key.append(e.key())
        for hotkey, func in self.hotkeys:
            if self.pressing_key == hotkey:
                func()
                return
        return super().keyPressEvent(e)

    def keyReleaseEvent(self, e: QKeyEvent) -> None:
        self.pressing_key.remove(e.key())
        return super().keyReleaseEvent(e)

    def registerHotkey(self, key: list, func):
        self.hotkeys.append((key, func))
