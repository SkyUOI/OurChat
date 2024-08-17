import json
from logging import getLogger
from typing import Union

from lib.const import USER_MSG
from peewee import IntegerField, Model, SqliteDatabase, TextField

logger = getLogger(__name__)


class SessionRecord(Model):
    msg_id = IntegerField(primary_key=True, null=False)
    time = IntegerField(null=False)
    msg = TextField(null=False)
    sender_ocid = TextField(null=False)
    read = IntegerField(null=False)


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
        self.readHavenotRead()
        self.ourchat.listen(USER_MSG, self.gotMessage)

    def createSessionTable(self, session: str) -> None:
        logger.info("create table")
        logger.debug(f"create table {session}")
        table = SessionRecord
        table._meta.table_name = session
        self.database.create_tables([table])

    def addRecord(self, session: str, data: dict) -> None:
        if not self.database.table_exists(session):
            self.createSessionTable(session)
        table = SessionRecord
        table._meta.table_name = session
        table.create(
            msg_id=data["msg_id"],
            time=data["time"],
            msg=json.dumps(data["msg"]),
            sender_ocid=data["sender"]["ocid"],
            read=False,
        )

    def getRecord(self, session: str, maximum=50, before=-1) -> list:
        if not self.database.table_exists(session):
            logger.warning("table not found")
            logger.debug(f"table {session} not found")
            return []
        table = SessionRecord
        table._meta.table_name = session
        query = table.select().order_by(SessionRecord.time.desc()).limit(maximum)
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
        self.addRecord(data["sender"]["session_id"], data)
        if data["sender"]["session_id"] not in self.havenot_read:
            self.havenot_read[data["sender"]["session_id"]] = 0
        self.havenot_read[data["sender"]["session_id"]] += 1

    def readSession(self, session: str) -> None:
        if not self.database.table_exists(session):
            return
        table = SessionRecord
        table._meta.table_name = session
        table.update({table.read: True}).where(table.read == 0).execute()
        self.havenot_read[session] = 0

    def readHavenotRead(self) -> None:
        tables = self.getSessions()
        for table_name in tables:
            table = SessionRecord
            table._meta.table_name = table_name
            self.havenot_read[table_name] = (
                table.select().where(table.read == 0).count()
            )

    def getSessions(self) -> Union[list, None]:
        return self.database.get_tables()

    def getHavenotReadNumber(self, session_id) -> int:
        if session_id not in self.havenot_read:
            return 0
        return self.havenot_read[session_id]
