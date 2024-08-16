import hashlib
import urllib.request
from logging import getLogger
from typing import Any

from lib.const import (
    SESSION_INFO_MSG,
    SESSION_INFO_RESPONSE_MSG,
)

logger = getLogger(__name__)


class OurChatSession:
    def __init__(
        self, ourchat, session_id: str, update_func: Any | None = None
    ) -> None:
        self.ourchat = ourchat
        self.session_id = session_id
        self.data = {}
        self.update_func = update_func
        self.request_values = [
            "name",
            "avatar",
            "avatar_hash",
            "time",
            "update_time",
            "members",
            "owner",
        ]
        self.ourchat.runThread(self.getInfo)

    def getAvatar(self) -> None:
        logger.info(f"get avatar(session_id:{self.session_id})")
        avatar_binary_data = self.ourchat.cache.getImage(self.data["avatar_hash"])
        if avatar_binary_data is None:
            logger.info("avatar cache not found,started to download")
            response = urllib.request.urlopen(self.data["avatar"])
            avatar_binary_data = response.read()
            logger.info("avatar download complete")
            sha256 = hashlib.sha256()
            sha256.update(avatar_binary_data)
            self.ourchat.cache.setImage(sha256.hexdigest(), avatar_binary_data)
        self.avatar_binary_data = avatar_binary_data
        if self.update_func is not None:
            self.update_func(self)

    def getInfo(self) -> None:
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

    def getInfoResponse(self, data: dict) -> None:
        self.ourchat.unListen(SESSION_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.data = data["data"]
        self.ourchat.cache.setSession(self.session_id, self.data)
        self.getAvatar()

    def sendInfoRequest(self) -> None:
        self.ourchat.listen(SESSION_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.ourchat.conn.send(
            {
                "code": SESSION_INFO_MSG,
                "session_id": self.session_id,
                "request_values": self.request_values,
            }
        )
