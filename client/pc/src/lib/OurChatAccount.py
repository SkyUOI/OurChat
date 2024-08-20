import hashlib
import json
from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    ACCOUNT_INFO_MSG,
    ACCOUNT_INFO_RESPONSE_MSG,
)
from PyQt6.QtWidgets import QMessageBox

logger = getLogger(__name__)


class OurChatAccount:
    def __init__(self, ourchat, ocid: str, me: bool = False) -> None:
        self.ourchat = ourchat
        self.ocid = ocid
        self.data = {}
        self.me = me
        self.avatar_binary_data = None
        self.request_values = [
            "nickname",
            "status",
            "avatar",
            "avatar_hash",
            "time",
            "public_update_time",
            "private_update_time",
        ]
        self.have_got_avatar = False
        self.have_got_info = False
        self.sessions = {}
        self.friends = {}
        if self.me:
            self.request_values.append("sessions")
            self.request_values.append("friends")
        self.ourchat.runThread(self.getInfo)

    def getAvatar(self) -> None:
        logger.info(f"get avatar(ocid:{self.ocid})")
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
            {"code": ACCOUNT_FINISH_GET_AVATAR, "ocid": self.ocid}
        )

    def getInfo(self) -> None:
        logger.info(f"get info(ocid:{self.ocid})")
        account_info = self.ourchat.cache.getAccount(self.ocid)
        if not self.me and account_info is not None:
            self.data = account_info
            self.ourchat.listen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            self.ourchat.conn.send(
                {
                    "code": ACCOUNT_INFO_MSG,
                    "ocid": self.ocid,
                    "request_values": ["public_update_time", "private_update_time"],
                }
            )
        else:
            self.sendInfoRequest()

    def getUpdateTimeResponse(self, data: dict) -> None:
        self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
        cache_update_time = self.data["public_update_time"]
        cloud_update_time = data["data"]["public_update_time"]
        if self.me:
            cache_update_time = self.data["private_update_time"]
            cloud_update_time = data["data"]["private_update_time"]
        if cache_update_time != cloud_update_time:
            self.sendInfoRequest()
        else:
            self.finishGetInfo()

    def getInfoResponse(self, data: dict) -> None:
        self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.data = data["data"]
        self.data["sessions"] = json.loads(self.data["sessions"])
        self.data["friends"] = json.loads(self.data["friends"])
        if not self.me:
            self.data["sessions"] = None
            self.data["friends"] = None
        self.ourchat.cache.setAccount(self.ocid, self.data)
        self.finishGetInfo()

    def sendInfoRequest(self) -> None:
        self.ourchat.listen(ACCOUNT_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.ourchat.conn.send(
            {
                "code": ACCOUNT_INFO_MSG,
                "ocid": self.ocid,
                "request_values": self.request_values,
            }
        )

    def finishGetInfo(self) -> None:
        if self.me:
            for session_id in self.data["sessions"]:
                self.sessions[session_id] = self.ourchat.getSession(session_id)
            for ocid in self.data["friends"]:
                self.friends[ocid] = self.ourchat.getAccount(ocid)
        else:
            self.sessions = None
            self.friends = None
        self.have_got_info = True
        self.ourchat.triggerEvent({"code": ACCOUNT_FINISH_GET_INFO, "ocid": self.ocid})
        self.getAvatar()
