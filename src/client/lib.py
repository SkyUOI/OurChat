import sqlite3


class Record:
    def __init__(self):
        self.db = None
        self.c = None
        self.chat_id = None
        self.table_data = []
        self.max_uid = 0

    def connectToDB(self):
        self.db = sqlite3.connect("record.db")
        self.cursor = self.db.cursor()

    def close(self):
        self.db.close()

    def createNewChatTable(self, chatId):
        command = f"""
		CREATE TABLE "{chatId}" (
			cid          INTEGER NOT NULL,
		    sender_json  TEXT    NOT NULL,
		    message_json TEXT    NOT NULL,
		    time         NUMERIC NOT NULL,
		    uid          INTEGER UNIQUE
		                         NOT NULL
		);
		"""
        self.cursor.execute(command)
        self.db.commit()

    def openTable(self, chat_id):
        self.chat_id = chat_id
        self.table_data = []
        self.max_uid = 0

        command = f'SELECT * FROM "{chat_id}";'
        data = self.cursor.execute(command)
        for row in data:
            cid, sender_ocid, message_json, time_, uid = row
            self.table_data.append(
                {
                    "cid": cid,
                    "sender_ocid": sender_ocid,
                    "message_json": message_json,
                    "time": time_,
                    "uid": uid,
                }
            )
            self.max_uid = max(self.max_uid, uid)
        self.table_data.sort(key=self.sortGetKey)

    def appendRecord(self, msg_data: dict):
        cid = msg_data["data"]["cid"]
        sender_json = msg_data["data"]["sender"]
        message_json = msg_data
        time_ = msg_data["time"]
        self.max_uid += 1
        uid = self.max_uid

        command = f"""INSERT INTO "{self.chat_id}" VALUES ({cid},"{sender_json}","{msg_data}",{time_},{uid})"""
        self.cursor.execute(command)
        self.db.commit()
        self.table_data.append(
            {
                "cid": cid,
                "sender_json": sender_json,
                "message_json": message_json,
                "time": time_,
                "uid": uid,
            }
        )

    def queryIntervalRecord(self, left, right):
        return self.table_data[left - 1, right]

    def sortGetKey(self, data):
        return data["uid"]

    def getTableList(self):
        table_list = []
        command = "select name from sqlite_master where type = 'table' order by name;"
        for name in self.cursor.execute(command):
            table_list.append(name[0])
        return table_list
