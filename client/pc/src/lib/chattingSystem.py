import json
from logging import getLogger

from lib.const import USER_MSG
from peewee import IntegerField, Model, SqliteDatabase, TextField

logger = getLogger(__name__)


class SessionRecord(Model):
    msg_id = IntegerField(primary_key=True, null=False)
    time = IntegerField(null=False)
    msg = TextField(null=False)
    sender_ocid = TextField(null=False)
    sender_session_id = TextField(null=False)
    read = IntegerField(null=False)

    class Meta:
        table_name = "record"


class ChattingSystem:
    def __init__(self, ourchat) -> None:
        logger.info("ChattingSystem init")
        self.ourchat = ourchat
        self.havenot_read = {}
        self.database = None

    def connectToDB(self, path: str = "record.db") -> None:
        logger.info(f"connect to chatting record database({path})")
        self.database = SqliteDatabase(path)
        SessionRecord._meta.database = self.database
        self.database.connect()
        SessionRecord.create_table(safe=True)
        self.ourchat.listen(USER_MSG, self.gotMessage)

    def addRecord(self, data: dict) -> None:
        SessionRecord.create(
            msg_id=data["msg_id"],
            time=data["time"],
            msg=json.dumps(data["msg"]),
            sender_ocid=data["sender"]["ocid"],
            sender_session_id=data["sender"]["session_id"],
            read=False,
        )

    def getRecord(self, session: str, maximum=50, before=-1) -> list:
        if not self.database.table_exists(SessionRecord):
            logger.warning("table not found")
            logger.debug(f"table {session} not found")
            return []
        query = (
            SessionRecord.select()
            .order_by(SessionRecord.time.desc())
            .limit(maximum)
            .where(SessionRecord.sender_session_id == session)
        )
        if before != -1:
            query = query.where(SessionRecord.time < before)
        data = []
        for row in query:
            data.append(
                {
                    "msg_id": row.msg_id,
                    "time": row.time,
                    "msg": json.loads(row.msg),
                    "sender": {"ocid": row.sender_ocid, "session_id": session},
                },
            )
        return data

    def close(self) -> None:
        logger.info("close chatting record database")
        if self.database is not None and not self.database.is_closed():
            self.database.close()

    def gotMessage(self, data) -> None:
        self.addRecord(data)
        if data["sender"]["session_id"] not in self.havenot_read:
            self.havenot_read[data["sender"]["session_id"]] = 0
        self.havenot_read[data["sender"]["session_id"]] += 1

    def readSession(self, session: str) -> None:
        if not self.database.table_exists(SessionRecord):
            return
        SessionRecord.update({SessionRecord.read: True}).where(
            SessionRecord.read == 0
        ).execute()
        self.havenot_read[session] = 0

    def getHavenotReadNumber(self, session_id) -> int:
        return (
            SessionRecord.select()
            .where(
                SessionRecord.sender_session_id == session_id
                and SessionRecord.read == 0
            )
            .count()
        )
