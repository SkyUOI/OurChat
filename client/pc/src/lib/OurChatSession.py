import hashlib
from logging import getLogger

from lib.const import (
    REQUEST_INFO_NOT_FOUND,
    RUN_NORMALLY,
    SERVER_ERROR,
    SERVER_UNDER_MAINTENANCE,
    SESSION_FINISH_GET_AVATAR,
    SESSION_FINISH_GET_INFO,
    SESSION_INFO_MSG,
    SESSION_INFO_RESPONSE_MSG,
    UNKNOWN_ERROR,
)
from PyQt6.QtWidgets import QMessageBox

logger = getLogger(__name__)


class OurChatSession:
    def __init__(self, ourchat, session_id: str) -> None:
        self.ourchat = ourchat
        self.session_id = session_id
        self.data = {}
        self.request_values = [
            "session_id",
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
        logger.info("get session avatar")
        logger.debug(f"get session avatar: {self.session_id}")
        if self.data["avatar"] is None:
            logger.info("avatar field is None,use default avatar")
            self.avatar_data = "resources/images/logo.png"
            return
        avatar_data = self.ourchat.cache.getImage(self.data["avatar_hash"])
        if avatar_data is None:
            logger.info("avatar cache not found,started to download")
            avatar_data = self.ourchat.download(self.data["avatar"])
            if avatar_data is None:
                self.ourchat.runInMainThread(
                    QMessageBox.warning(
                        None,
                        self.ourchat.language["warning"],
                        self.ourchat.language["avatar_download_failed"],
                    )
                )
            logger.info("avatar download complete")
            sha256 = hashlib.sha256()
            sha256.update(avatar_data)
            self.ourchat.cache.setImage(sha256.hexdigest(), avatar_data)
        self.avatar_data = avatar_data
        self.have_got_avatar = True
        self.ourchat.triggerEvent(
            {"code": SESSION_FINISH_GET_AVATAR, "session_id": self.session_id}
        )

    def getInfo(self) -> None:
        logger.info("get session info")
        logger.debug(f"get session info: {self.session_id}")
        session_info = self.ourchat.cache.getSession(self.session_id)
        if session_info is not None:
            self.data = session_info
            self.ourchat.listen(SESSION_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            self.ourchat.conn.send(
                {
                    "code": SESSION_INFO_MSG,
                    "session_id": self.session_id,
                    "request_values": ["session_id", "update_time"],
                }
            )
        else:
            self.sendInfoRequest()

    def getUpdateTimeResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            if data["data"]["session_id"] != self.session_id:
                return
            self.ourchat.unListen(SESSION_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            update_time = data["data"]["update_time"]
            if self.data["update_time"] != update_time:
                self.sendInfoRequest()
            else:
                self.finishGetInfo()
        elif data["status"] == SERVER_ERROR:
            logger.warning("get session info failed: server error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("get session info failed: server under maintenance")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == REQUEST_INFO_NOT_FOUND:
            logger.warning("get session info failed: session not found")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["session_not_found"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("get session info failed: unknown error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )

    def getInfoResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            if data["data"]["session_id"] != self.session_id:
                return
            self.ourchat.unListen(SESSION_INFO_RESPONSE_MSG, self.getInfoResponse)
            self.data = data["data"]
            self.ourchat.cache.setSession(self.session_id, self.data)
            self.finishGetInfo()
        elif data["status"] == SERVER_ERROR:
            logger.warning("get session info failed: server error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("get session info failed: server under maintenance")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == REQUEST_INFO_NOT_FOUND:
            logger.warning("get session info failed: session not found")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["session_not_found"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("get session info failed: unknown error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_session_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )

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
        if self.data["name"] is None:
            self.data["name"] = self.ourchat.language["default_session_name"]
        self.ourchat.triggerEvent(
            {"code": SESSION_FINISH_GET_INFO, "session_id": self.session_id}
        )
        self.getAvatar()
