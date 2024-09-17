import time
from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    DEFAULT_IMAGE,
    SESSION_FINISH_GET_AVATAR,
    SESSION_FINISH_GET_INFO,
    USER_MSG,
)
from lib.OurChatSession import OurChatSession
from lib.OurChatUI import (
    MessageListItemWidget,
    OurChatEditor,
    OurChatWidget,
    SessionListItemWidget,
)
from PyQt6.QtCore import QSize, Qt
from PyQt6.QtWidgets import QListWidgetItem
from ui.session import Ui_Session
from ui_logic.basicUI import BasicUI
from ui_logic.sessionSetting import SessionSettingUI

logger = getLogger(__name__)


class SessionUI(BasicUI, Ui_Session):
    def __init__(self, ourchat, widget: OurChatWidget) -> None:
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.widget = widget
        self.sessions = {}
        self.messages = []

    def setupUi(self) -> None:
        logger.info("setup Ui")
        super().setupUi(self.widget)
        self.editor.deleteLater()
        self.editor = OurChatEditor(parent=self.widget)
        self.right_panel.setStretch(0, 1)
        self.right_panel.setStretch(1, 10)
        self.right_panel.setStretch(2, 6)
        self.right_panel.setStretch(3, 1)
        self.editor.setReadOnly(False)
        self.editor.registerHotkey(
            [Qt.Key.Key_Control, Qt.Key.Key_Return], self.newline
        )
        self.editor.registerHotkey([Qt.Key.Key_Return], self.send_btn.click)
        self.right_panel.insertWidget(2, self.editor)
        self.message_list.verticalScrollBar().setSingleStep(10)
        self.listen()
        self.fillText()
        self.bind()

    def fillText(self) -> None:
        self.widget.setWindowTitle(f"Ourchat - {self.ourchat.language['session']}")
        self.send_btn.setEnabled(False)
        self.title.setText("")
        self.editor.setEnabled(False)
        self.send_btn.setText(self.ourchat.language["send"])
        self.add_session_btn.setText("+")
        if self.ourchat.account.have_got_info:
            self.updateSessionList()

    def addSessions(self, sessions: list):
        self.session_list.clear()
        self.sessions = {}
        for session in sessions:
            self.addSessionItem(session)

    def bind(self) -> None:
        self.session_list.itemClicked.connect(self.openSession)
        self.send_btn.clicked.connect(self.send)
        self.add_session_btn.clicked.connect(self.createSession)
        self.search_box.textChanged.connect(self.updateSessionList)

    def addSessionItem(self, session: OurChatSession) -> None:
        recent_msg = self.ourchat.chatting_system.getRecord(session.session_id, 1)
        avatar = DEFAULT_IMAGE
        name = session.session_id
        recent_msg_text = ""

        if session.have_got_avatar:
            avatar = session.avatar_data
        if session.have_got_info:
            name = session.data["name"]
        if len(recent_msg) >= 1:
            recent_msg_text = recent_msg[0]["msg"][0]["text"]

        have_not_read = self.ourchat.chatting_system.getHavenotReadNumber(
            session.session_id
        )
        item = QListWidgetItem(self.session_list)
        item.setSizeHint(QSize(65, 65))
        widget = SessionListItemWidget(self.session_list)
        widget.setSession(
            session.session_id,
            avatar,
            name,
            f"{f'[{have_not_read}] ' if have_not_read > 0 else ''}{recent_msg_text[:10]}{'...' if len(recent_msg_text)>10 else ''}",
        )
        self.session_list.addItem(item)
        self.session_list.setItemWidget(item, widget)
        self.sessions[session.session_id] = widget

    def addMessageItem(self, message) -> None:
        item = QListWidgetItem(self.message_list)
        self.message_list.addItem(item)
        widget = MessageListItemWidget(self.message_list)
        widget.ocid = message["sender"]["ocid"]
        sender_account = self.ourchat.getAccount(message["sender"]["ocid"])
        avatar = DEFAULT_IMAGE
        name = message["sender"]["ocid"]

        account = self.ourchat.getAccount(message["sender"]["ocid"])
        if account.have_got_info:
            name = account.data["nickname"]
        if account.have_got_avatar:
            avatar = account.avatar_data

        widget.setMessage(
            item,
            avatar,
            name,
            message["msg"][0]["text"],
            sender_account == self.ourchat.account,
        )
        self.message_list.setItemWidget(item, widget)
        self.message_list.setCurrentItem(item)
        self.messages.append(widget)

    def openSession(self, item: QListWidgetItem) -> None:
        session_id = self.session_list.itemWidget(item).session_id
        self.messages.clear()
        self.message_list.clear()
        self.updateSessionWidget(session_id)
        self.title.setText(self.ourchat.getSession(session_id).data["name"])
        self.send_btn.setEnabled(True)
        self.editor.setEnabled(True)
        self.editor.clear()

    def insertMessages(self, messages) -> None:
        for message in messages:
            self.addMessageItem(message)

    def getAccountInfoResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        if account == self.ourchat.account:
            self.updateSessionList()
        for message in self.messages:
            if message.ocid == account.ocid:
                message.setName(account.data["nickname"])
        self.updateSessionList()

    def getAccountAvatarResponse(self, data: dict) -> None:
        account = self.ourchat.getAccount(data["ocid"])
        for message in self.messages:
            if message.ocid == account.ocid:
                message.setAvatar(account.avatar_data)

    def getSessionInfoResponse(self, data: dict) -> None:
        session = self.ourchat.getSession(data["session_id"])
        if session.session_id not in self.sessions:
            return
        session_widget = self.sessions[session.session_id]
        session_widget.setName(session.data["name"])

    def getSessionAvatarResponse(self, data: dict) -> None:
        session = self.ourchat.getSession(data["session_id"])
        if session.session_id not in self.sessions:
            return
        session_widget = self.sessions[session.session_id]
        session_widget.setAvatar(session.avatar_data)

    def send(self):
        text = self.editor.toPlainText()
        session_id = self.session_list.itemWidget(
            self.session_list.currentItem()
        ).session_id
        self.ourchat.conn.send(
            {
                "code": USER_MSG,
                "time": time.time(),
                "sender": {"session_id": session_id},
                "msg": [{"type": 0, "text": text}],
            }
        )
        self.editor.clear()

    def messageResponse(self, data: dict) -> None:
        self.updateSessionWidget(data["sender"]["session_id"])

    def updateSessionWidget(self, session_id: str) -> None:
        current_session_widget = self.session_list.itemWidget(
            self.session_list.currentItem()
        )
        if (
            current_session_widget is not None
            and current_session_widget.session_id == session_id
        ):
            record = self.ourchat.chatting_system.getRecord(
                session_id,
                self.ourchat.chatting_system.getHavenotReadNumber(session_id)
                if len(self.messages)
                else 50,
            )
            record.reverse()
            for message in record:
                self.addMessageItem(message)
            self.ourchat.chatting_system.readSession(current_session_widget.session_id)
        have_not_read = self.ourchat.chatting_system.getHavenotReadNumber(session_id)
        message_record = self.ourchat.chatting_system.getRecord(session_id, 1)
        if len(message_record) >= 1:
            recent_msg_text = message_record[0]["msg"][0]["text"]
            session_widget = self.sessions[session_id]
            session_widget.setDetail(
                f"{f'[{have_not_read}] ' if have_not_read > 0 else ''}{recent_msg_text[:10]}{'...' if len(recent_msg_text)>10 else ''}",
            )

    def listen(self):
        self.ourchat.listen(SESSION_FINISH_GET_INFO, self.getSessionInfoResponse)
        self.ourchat.listen(SESSION_FINISH_GET_AVATAR, self.getSessionAvatarResponse)
        self.ourchat.listen(USER_MSG, self.messageResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        self.ourchat.listen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse)

    def createSession(self):
        dialog = self.uisystem.setDialog(SessionSettingUI, True)
        dialog.show()

    def newline(self):
        self.editor.insertPlainText("\n")

    def updateSessionList(self):
        result = []
        keyword = self.search_box.text()
        if keyword == "":
            self.addSessions(
                [
                    self.ourchat.getSession(session_id)
                    for session_id in self.ourchat.account.sessions
                ]
            )
            return None
        for session_id in self.ourchat.account.sessions:
            session = self.ourchat.getSession(session_id)
            if keyword in session.data["name"] or keyword in session_id:
                result.append(session)
        self.addSessions(result)

    def close(self):
        self.ourchat.unListen(SESSION_FINISH_GET_INFO, self.getSessionInfoResponse)
        self.ourchat.unListen(SESSION_FINISH_GET_AVATAR, self.getSessionAvatarResponse)
        self.ourchat.unListen(USER_MSG, self.messageResponse)
        self.ourchat.unListen(ACCOUNT_FINISH_GET_INFO, self.getAccountInfoResponse)
        self.ourchat.unListen(ACCOUNT_FINISH_GET_AVATAR, self.getAccountAvatarResponse)
