import json
from logging import getLogger
from typing import Union

from peewee import BlobField, IntegerField, Model, SqliteDatabase, TextField

logger = getLogger(__name__)


class ImageCache(Model):
    image_hash = TextField(null=False, primary_key=True)
    image_data = BlobField(null=False)

    class Meta:
        table_name = "image_cache"


class AccountCache(Model):
    ocid = TextField(null=False, primary_key=True)
    nickname = TextField(null=False)
    status = IntegerField(null=False)
    avatar = TextField(null=False)
    avatar_hash = TextField(null=False)
    time = IntegerField(null=False)
    public_update_time = IntegerField(null=False)
    update_time = IntegerField(null=False)

    class Meta:
        table_name = "account_cache"


class SessionCache(Model):
    session_id = TextField(null=False, primary_key=True)
    name = TextField(null=True)
    avatar = TextField(null=True)
    avatar_hash = TextField(null=True)
    time = IntegerField(null=False)
    update_time = IntegerField(null=False)
    members = TextField(null=False)
    owner = TextField(null=False)

    class Meta:
        table_name = "session_cache"


class OurChatCache:
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat

    def connectToDB(self, path: str = "cache.db") -> None:
        logger.info(f"connect to cache database({path})")
        self.database = SqliteDatabase(path)
        AccountCache._meta.database = self.database
        ImageCache._meta.database = self.database
        SessionCache._meta.database = self.database
        self.database.connect()
        for table in [AccountCache, ImageCache, SessionCache]:
            table.create_table(safe=True)

    def getImage(self, image_hash: str) -> Union[None, bytes]:
        image = ImageCache.get_or_none(ImageCache.image_hash == image_hash)
        if image is None:
            return None
        return image.image_data

    def getAccount(self, ocid: str) -> Union[None, dict]:
        account_info = AccountCache.get_or_none(AccountCache.ocid == ocid)
        if account_info is None:
            return None
        return {
            "ocid": ocid,
            "nickname": account_info.nickname,
            "status": account_info.status,
            "avatar": account_info.avatar,
            "avatar_hash": account_info.avatar_hash,
            "time": account_info.time,
            "public_update_time": account_info.public_update_time,
            "update_time": account_info.update_time,
        }

    def getSession(self, session_id: str) -> Union[None, dict]:
        session_info = SessionCache.get_or_none(SessionCache.session_id == session_id)
        if session_info is None:
            return None
        return {
            "session_id": session_id,
            "name": session_info.name,
            "avatar": session_info.avatar,
            "avatar_hash": session_info.avatar_hash,
            "time": session_info.time,
            "update_time": session_info.update_time,
            "members": json.loads(session_info.members),
            "owner": json.loads(session_info.owner),
        }

    def setImage(self, image_hash: str, image_data: bytes) -> None:
        if ImageCache.get_or_none(ImageCache.image_hash == image_hash) is None:
            ImageCache.create(image_hash=image_hash, image_data=image_data)
        else:
            ImageCache.update(image_hash=image_hash, image_data=image_data).where(
                ImageCache.image_hash == image_hash
            ).execute()

    def setAccount(self, ocid: str, data: dict) -> None:
        if AccountCache.get_or_none(AccountCache.ocid == ocid) is None:
            AccountCache.create(
                ocid=ocid,
                nickname=data["nickname"],
                status=data["status"],
                avatar=data["avatar"],
                avatar_hash=data["avatar_hash"],
                time=data["time"],
                public_update_time=data["public_update_time"],
                update_time=data["update_time"],
            )
        else:
            AccountCache.update(
                ocid=ocid,
                nickname=data["nickname"],
                status=data["status"],
                avatar=data["avatar"],
                avatar_hash=data["avatar_hash"],
                time=data["time"],
                public_update_time=data["public_update_time"],
                update_time=data["update_time"],
            ).where(AccountCache.ocid == ocid).execute()

    def setSession(self, session_id: str, data: dict) -> None:
        if SessionCache.get_or_none(SessionCache.session_id == session_id) is None:
            SessionCache.create(
                session_id=session_id,
                name=data["name"],
                avatar=data["avatar"],
                avatar_hash=data["avatar_hash"],
                time=data["time"],
                update_time=data["update_time"],
                members=json.dumps(data["members"]),
                owner=json.dumps(data["owner"]),
            )
        else:
            SessionCache.update(
                session_id=session_id,
                name=data["name"],
                avatar=data["avatar"],
                avatar_hash=data["avatar_hash"],
                time=data["time"],
                update_time=data["update_time"],
                members=json.dumps(data["members"]),
                owner=json.dumps(data["owner"]),
            ).where(SessionCache.session_id == session_id).execute()

    def close(self) -> None:
        logger.info("close cache database")
        self.database.close()
