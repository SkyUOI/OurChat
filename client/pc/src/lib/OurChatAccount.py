import json
from logging import getLogger

from lib.const import (
    ACCOUNT_FINISH_GET_AVATAR,
    ACCOUNT_FINISH_GET_INFO,
    ACCOUNT_INFO_MSG,
    ACCOUNT_INFO_RESPONSE_MSG,
    DOWNLOAD_RESPONSE,
    REQUEST_INFO_NOT_FOUND,
    RUN_NORMALLY,
    SERVER_ERROR,
    SERVER_UNDER_MAINTENANCE,
    UNKNOWN_ERROR,
)
from PyQt6.QtWidgets import QMessageBox

logger = getLogger(__name__)


class OurChatAccount:
    def __init__(self, ourchat, ocid: str, me: bool = False) -> None:
        self.ourchat = ourchat
        self.ocid = ocid
        self.data = {}
        self.me = me
        self.avatar_data = None
        self.request_values = [
            "ocid",
            "nickname",
            "status",
            "avatar",
            "time",
            "public_update_time",
            "update_time",
        ]
        self.have_got_avatar = False
        self.have_got_info = False
        self.sessions = []
        self.friends = []
        if self.me:
            self.request_values.append("sessions")
            self.request_values.append("friends")
        self.ourchat.runThread(self.getInfo)

    def getAvatar(self) -> None:
        logger.info("get account avatar")
        logger.debug(f"get account avatar: {self.ocid}")
        avatar_data = self.ourchat.cache.getImage(self.data["avatar"])
        if avatar_data is None:
            logger.info("avatar cache not found,started to download")
            self.ourchat.listen(DOWNLOAD_RESPONSE, self.downloadAvatarResponse)
            self.ourchat.download(self.data["avatar"])
        else:
            self.finishGetAvatar(avatar_data)

    def downloadAvatarResponse(self, data: dict):
        if data["url"] != self.data["avatar"]:
            return
        self.ourchat.unListen(DOWNLOAD_RESPONSE, self.downloadAvatarResponse)
        if data["status"] == RUN_NORMALLY:
            logger.info("avatar download complete")
            self.ourchat.cache.setImage(self.data["avatar"], data["data"])
            self.finishGetAvatar(data["data"])
        elif data["status"] == REQUEST_INFO_NOT_FOUND:
            self.ourchat.runInMainThread(
                lambda: QMessageBox.warning(
                    None,
                    self.ourchat.language["warning"],
                    self.ourchat.language["avatar_download_failed"],
                )
            )
            return

    def finishGetAvatar(self, avatar_data: bytes):
        self.avatar_data = avatar_data
        self.have_got_avatar = True
        self.ourchat.triggerEvent(
            {"code": ACCOUNT_FINISH_GET_AVATAR, "ocid": self.ocid}
        )

    def getInfo(self) -> None:
        logger.info("get account info")
        logger.debug(f"get account info: {self.ocid}")
        account_info = self.ourchat.cache.getAccount(self.ocid)
        if not self.me and account_info is not None:
            self.data = account_info
            self.ourchat.listen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            self.ourchat.conn.send(
                {
                    "code": ACCOUNT_INFO_MSG,
                    "ocid": self.ocid,
                    "request_values": ["ocid", "public_update_time", "update_time"],
                }
            )
        else:
            self.sendInfoRequest()

    def getUpdateTimeResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            if data["data"]["ocid"] != self.ocid:
                return
            self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getUpdateTimeResponse)
            cache_update_time = self.data["public_update_time"]
            cloud_update_time = data["data"]["public_update_time"]
            if self.me:
                cache_update_time = self.data["update_time"]
                cloud_update_time = data["data"]["update_time"]
            if cache_update_time != cloud_update_time:
                self.sendInfoRequest()
            else:
                self.finishGetInfo()
        elif data["status"] == SERVER_ERROR:
            logger.warning("get account info failed: server error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("get account info failed: server under maintenance")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == REQUEST_INFO_NOT_FOUND:
            logger.warning("get account info failed: account not found")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["account_not_found"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("get account info failed: unknown error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )

    def getInfoResponse(self, data: dict) -> None:
        if data["status"] == RUN_NORMALLY:
            if data["data"]["ocid"] != self.ocid:
                return
            self.ourchat.unListen(ACCOUNT_INFO_RESPONSE_MSG, self.getInfoResponse)
            self.data = data["data"]
            if self.me:
                self.data["sessions"] = json.loads(self.data["sessions"])
                self.data["friends"] = json.loads(self.data["friends"])
            else:
                self.data["sessions"] = None
                self.data["friends"] = None
            self.ourchat.cache.setAccount(self.ocid, self.data)
            self.finishGetInfo()
        elif data["status"] == SERVER_ERROR:
            logger.warning("get account info failed: server error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["server_error"]
                ),
            )
        elif data["status"] == SERVER_UNDER_MAINTENANCE:
            logger.warning("get account info failed: server under maintenance")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["maintenance"]
                ),
            )
        elif data["status"] == REQUEST_INFO_NOT_FOUND:
            logger.warning("get account info failed: account not found")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["account_not_found"]
                ),
            )
        elif data["status"] == UNKNOWN_ERROR:
            logger.warning("get account info failed: unknown error")
            QMessageBox.warning(
                None,
                self.ourchat.language["warning"],
                self.ourchat.language["get_account_info_failed"].format(
                    self.ourchat.language["unknown_error"]
                ),
            )

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
            self.sessions = []
            self.friends = []
            for session_id in self.data["sessions"]:
                self.ourchat.getSession(session_id)
                self.sessions.append(session_id)
            for ocid in self.data["friends"]:
                self.ourchat.getAccount(ocid)
                self.friends.append(ocid)
        else:
            self.sessions = None
            self.friends = None
        self.have_got_info = True
        self.ourchat.triggerEvent({"code": ACCOUNT_FINISH_GET_INFO, "ocid": self.ocid})
        # self.getAvatar()
