import hashlib
import urllib.request
from logging import getLogger
from typing import Any

from lib.const import ACCOUNT_INFO_MSG, ACCOUNT_INFO_RESPONSE_MSG

logger = getLogger(__name__)


class OurChatAccount:
    def __init__(
        self, ourchat, ocid: str, update_func: Any | None = None, me: bool = False
    ) -> None:
        self.ourchat = ourchat
        self.ocid = ocid
        self.data = {}
        self.update_func = update_func
        self.me = me
        self.avatar_binary_data = None
        self.request_values = [
            "nickname",
            "status",
            "avatar",
            "avatar_hash",
            "time",
            "update_time",
        ]
        if self.me:
            self.request_values.append("sessions")
            self.request_values.append("friends")
        self.ourchat.runThread(self.getInfo)

    def getAvatar(self) -> None:
        logger.info(f"get avatar(ocid:{self.ocid})")
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
        logger.info(f"get info(ocid:{self.ocid})")
        account_info = self.ourchat.cache.getAccount(self.ocid)
        if account_info is not None:
            self.data = account_info
            self.ourchat.listen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            self.ourchat.conn.send(
                {
                    "code": ACCOUNT_INFO_MSG,
                    "ocid": self.ocid,
                    "request_values": ["update_time"],
                }
            )
        else:
            self.sendInfoRequest()

    def getUpdateTimeResponse(self, data: dict) -> None:
        self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
        update_time = data["data"]["update_time"]
        if self.data["update_time"] != update_time:
            self.sendInfoRequest()
        else:
            self.getAvatar()

    def getInfoResponse(self, data: dict) -> None:
        self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.data = data["data"]
        if not self.me:
            self.data["sessions"] = None
            self.data["friends"] = None
        self.ourchat.cache.setAccount(self.ocid, self.data)
        self.getAvatar()

    def sendInfoRequest(self) -> None:
        self.ourchat.listen(ACCOUNT_INFO_RESPONSE_MSG, self.getInfoResponse)
        self.ourchat.conn.send(
            {
                "code": ACCOUNT_INFO_MSG,
                "ocid": self.ocid,
                "request_values": self.request_values,
            }
        )
