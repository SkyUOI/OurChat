from peewee import SqliteDatabase, Model, IntegerField, TextField
from logging import getLogger
from lib.const import USER_MSG

logger = getLogger(__name__)


class SessionRecord(Model):
    msg_id = IntegerField(primary_key=True, null=False)
    time = IntegerField(null=False)
    msg = TextField(null=False)
    sender_ocid = TextField(null=False)


class ChattingSystem:
    def __init__(self, ourchat):
        logger.info("ChattingSystem init")
        self.ourchat = ourchat
        self.uisystem = self.ourchat.uisystem
        self.ourchat.listen(USER_MSG, self.gotMessage)

    def connectToDB(self, path: str = "record.db"):
        logger.info(f"connect to chatting record datebase({path})")
        self.datebase = SqliteDatabase(path)
        SessionRecord._meta.database = self.datebase
        self.datebase.connect()

    def createSessionTable(self, session: str):
        logger.info("create table")
        logger.debug(f"create table {session}")
        table = SessionRecord
        table._meta.table_name = session
        self.datebase.create_tables([table])

    def addRecord(self, session: str, data: dict):
        if not self.datebase.table_exists(session):
            self.createSessionTable(session)
        table = SessionRecord
        table._meta.table_name = session
        table.create(
            msg_id=data["msg_id"],
            time=data["time"],
            msg=data["msg"],
            sender_ocid=data["sender"]["ocid"],
        )

    def getRecord(self, session: str, maximum=50, before=-1):
        table = SessionRecord
        table._meta.table_name = session
        query = table.select().order_by(SessionRecord.time.desc()).limit(maximum)
        if before != -1:
            query = query.where(SessionRecord.time < before)
        data = []
        for row in query:
            data.insert(
                0,
                {
                    "msg_id": row.msg_id,
                    "time": row.time,
                    "msg": row.msg,
                    "sender": {"ocid": row.sender_ocid, "session_id": session},
                },
            )
        return data

    def close(self):
        logger.info("close chatting record datebase")
        self.datebase.close()

    def gotMessage(self, data):
        self.addRecord(data["sender"]["session_id"], data)
