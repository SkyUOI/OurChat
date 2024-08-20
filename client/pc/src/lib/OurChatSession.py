import hashlib
from logging import getLogger

from lib.const import (
    SESSION_FINISH_GET_AVATAR,
    SESSION_FINISH_GET_INFO,
    SESSION_INFO_MSG,
    SESSION_INFO_RESPONSE_MSG,
)
from PyQt6.QtWidgets import QMessageBox

logger = getLogger(__name__)


class OurChatSession:
    def __init__(self, ourchat, session_id: str) -> None:
        self.ourchat = ourchat
        self.session_id = session_id
        self.data = {}
        self.request_values = [
            "name",
            "avatar",
            "avatar_hash",
            "time",
            "update_time",
            "members",
            "owner",
        ]
        self.have_got_avatar = False
        self.have_got_info = False
        self.ourchat.runThread(self.getInfo)

    def getAvatar(self, depth: int = 0) -> None:
        if depth >= 5:
            return
        logger.info(f"get avatar(session_id:{self.session_id})")
        avatar_binary_data = self.ourchat.cache.getImage(self.data["avatar_hash"])
        if avatar_binary_data is None:
            logger.info("avatar cache not found,started to download")
            avatar_binary_data = self.ourchat.download(self.data["avatar"])
            if avatar_binary_data is None:
                self.ourchat.runInMainThread(
                    QMessageBox.warning(
                        None,
                        self.ourchat.language["warning"],
                        self.ourchat.language["avatar_download_failed"],
                    )
                )
            logger.info("avatar download complete")
            sha256 = hashlib.sha256()
            sha256.update(avatar_binary_data)
            self.ourchat.cache.setImage(sha256.hexdigest(), avatar_binary_data)
        self.avatar_binary_data = avatar_binary_data
        self.have_got_avatar = True
        self.ourchat.triggerEvent(
            {"code": SESSION_FINISH_GET_AVATAR, "session_id": self.session_id}
        )

    def getInfo(self) -> None:
        logger.info(f"get info(session_id:{self.session_id})")
        session_info = self.ourchat.cache.getSession(self.session_id)
        if session_info is not None:
            self.data = session_info
            self.ourchat.listen(SESSION_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            self.ourchat.conn.send(
                {
                    "code": SESSION_INFO_MSG,
                    "session_id": self.session_id,
                    "request_values": ["update_time"],
                }
            )
        else:
            self.sendInfoRequest()

    def getUpdateTimeResponse(self, data: dict) -> None:
        self.ourchat.unListen(SESSION_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
        update_time = data["data"]["update_time"]
        if self.data["update_time"] != update_time:
            self.sendInfoRequest()
        else:
            self.finishGetInfo()

    def getInfoResponse(self, data: dict) -> None:
        self.ourchat.unListen(SESSION_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.data = data["data"]
        self.ourchat.cache.setSession(self.session_id, self.data)
        self.finishGetInfo()

    def sendInfoRequest(self) -> None:
        self.ourchat.listen(SESSION_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.ourchat.conn.send(
            {
                "code": SESSION_INFO_MSG,
                "session_id": self.session_id,
                "request_values": self.request_values,
            }
        )

    def finishGetInfo(self) -> None:
        self.have_got_info = True
        self.ourchat.triggerEvent(
            {"code": SESSION_FINISH_GET_INFO, "session_id": self.session_id}
        )
        self.getAvatar()
