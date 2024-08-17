from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_INFO,
    SESSION_FINISH_GET_AVATAR,
    SESSION_FINISH_GET_INFO,
)
from lib.OurChatSession import OurChatSession
from lib.OurChatUI import OurChatWidget, SessionWidget
from PyQt6.QtCore import QSize
from PyQt6.QtWidgets import QListWidgetItem
from ui.session import Ui_Session

logger = getLogger(__name__)


class SessionUI(Ui_Session):
    def __init__(self, ourchat, widget: OurChatWidget) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget
        self.sessions = {}

    def setupUi(self) -> None:
        logger.info("setup Ui")
        super().setupUi(self.widget)
        self.message_list.verticalScrollBar().setSingleStep(10)
        self.fillText()
        self.bind()

    def fillText(self) -> None:
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")
        self.send_btn.setEnabled(False)
        self.title.setText("")
        self.editor.setEnabled(False)
        self.send_btn.setText(self.ourchat.language["send"])
        self.addSessions()

    def addSessions(self):
        if self.ourchat.account.have_got_info:
            for session in self.ourchat.account.sessions:
                self.addSession(self.ourchat.account.sessions[session])
            self.ourchat.listen(SESSION_FINISH_GET_INFO, self.getSessionInfoResponse)
            self.ourchat.listen(
                SESSION_FINISH_GET_AVATAR, self.getSessionAvatarResponse
            )
        else:
            self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)

    def bind(self) -> None:
        self.session_list.itemClicked.connect(self.openSession)

    def addSession(self, session: OurChatSession) -> None:
        recent_msg = self.ourchat.chatting_system.getRecord(session.session_id, 1)
        avatar = "resources/images/logo.png"
        name = session.session_id
        recent_msg_text = ""

        if session.have_got_avatar:
            avatar = session.avatar_binary_data
        if session.have_got_info:
            name = session.data["name"]
        if len(recent_msg) >= 1:
            recent_msg_text = recent_msg[0]["text"]

        item = QListWidgetItem(self.session_list)
        item.setSizeHint(QSize(65, 65))
        widget = SessionWidget(self.session_list)
        widget.setSession(
            session.session_id,
            avatar,
            name,
            recent_msg_text,
        )
        self.session_list.addItem(item)
        self.session_list.setItemWidget(item, widget)
        self.sessions[session.session_id] = widget

    def addMessage(self, message) -> None:
        pass

    def openSession(self, item: QListWidgetItem) -> None:
        session_id = self.session_list.itemWidget(item).session_id
        self.ourchat.chatting_system.readSession(session_id)
        self.title.setText(self.ourchat.getSession(session_id).data["name"])
        self.send_btn.setEnabled(True)
        self.editor.setEnabled(True)
        self.editor.clear()

    def insertMessages(self, messages) -> None:
        for message in messages:
            self.addMessage(message)

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.ourchat.unListen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
            self.addSessions()

    def getSessionInfoResponse(self, data: dict) -> None:
        session = self.ourchat.getSession(data["session_id"])
        session_widget = self.sessions[session.session_id]
        session_widget.setName(session.data["name"])

    def getSessionAvatarResponse(self, data: dict) -> None:
        session = self.ourchat.getSession(data["session_id"])
        session_widget = self.sessions[session.session_id]
        session_widget.setAvatar(session.avatar_binary_data)
