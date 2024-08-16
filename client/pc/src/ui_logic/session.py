from logging import getLogger

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
        self.sessions = []

    def setupUi(self) -> None:
        logger.info("setup Ui")
        super().setupUi(self.widget)
        self.message_list.verticalScrollBar().setSingleStep(10)
        self.fillText()
        self.bind()

        self.session_list.clear()
        self.message_list.clear()

    def sessionUpdateResponse(self, session: OurChatSession) -> None:
        print(session.session_id)
        self.addSession(session)

    def fillText(self) -> None:
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")

    def bind(self) -> None:
        self.session_list.itemClicked.connect(self.openSession)

    def addSession(self, session: OurChatSession) -> None:
        recent_msg = self.ourchat.chatting_system.getRecord(session.session_id, 1)[0]
        user_msg = recent_msg["msg"]

        item = QListWidgetItem(self.session_list)
        item.setSizeHint(QSize(65, 65))
        widget = SessionWidget(self.session_list)
        widget.setSession(
            session.session_id,
            session.avatar,
            session.data["name"],
            user_msg[0]["text"],
        )
        self.session_list.addItem(item)
        self.session_list.setItemWidget(item, widget)
        self.sessions.append(session.session_id)

    def addMessage(self, message) -> None:
        pass

    def openSession(self, item: QListWidgetItem) -> None:
        session_id = self.session_list.itemWidget(item).session_id
        self.ourchat.chatting_system.readSession(session_id)
        print(self.ourchat.chatting_system.getRecord(session_id))

    def updateSession(self) -> None:
        for session_id in self.ourchat.chatting_system.getSessions():
            if session_id not in self.sessions:
                recent_msg = self.ourchat.chatting_system.getRecord(session_id, 1)[0]
                user_msg = recent_msg["msg"]
                self.addSession(
                    session_id,
                    "resources/images/senlinjun.jpg",
                    session_id,
                    f"[{self.ourchat.chatting_system.havenot_read[session_id]}条] {user_msg[0]['text'][:5]}{'...' if len(user_msg[0]['text']) > 5 else ''}",
                )

    def insertMessages(self, messages) -> None:
        for message in messages:
            self.addMessage(message)
