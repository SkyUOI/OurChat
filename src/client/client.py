from config.const import *
from threading import Thread

import socket, json,hashlib,time,sqlite3,os


class Client(Thread):
    def __init__(self):
        self.ip = self.getIp()
        self.server_ip = ""
        self.s = None
        self.ui = None
        self.connect_to_server = False
        self.ocid = ""
        super().__init__()

    def getIp(self):
        return socket.gethostbyname(socket.gethostname())

    def setServerIp(self, ip):
        self.server_ip = ip

    def setUi(self, ui):

        self.ui = ui

    def run(self):
        self.connectToHost()

    def connectToHost(self):
        while True:
            try:
                self.s = socket.socket()
                self.s.connect((self.server_ip, SERVERPORT))
                self.startListen()
            except Exception as e:
                print(e)
                self.connect_to_server = False
                if self.ui != None:
                    self.ui.func_queue.append(self.ui.CantConnectServer.show)

    def sendByte(self, byte):
        if self.connect_to_server:
            self.s.send(byte)
        else:
            return False

    def startListen(self):
        if self.ui != None:
            self.ui.func_queue.append(self.ui.CantConnectServer.hide)
        self.connect_to_server = True
        print("connect to server")
        while True:
            recv = self.s.recv(RECVMAX)
            if recv.decode(ENCODE) == "":
                continue
            json_data = recv.decode(ENCODE)
            data = json.loads(json_data)
            
            if data[KCODE] == NORMALCODE:
                self.addRecord(data)
            
            elif data[KCODE] == REGISTERRETURNCODE:
                if data[KDATA]["state"] == 0:
                    self.ocid = data[KDATA]["ocId"]
                    self.ui.func_queue.append(self.ui.showChat)

                elif data[KDATA]["state"] == 1:
                    self.ui.func_queue.append(lambda:self.ui.setLoginTip(self.ui.lang[19]))
            
            elif data[KCODE] == LOGINRETURNCODE:
                if data[KDATA]["state"] == 0:
                    self.ocid = self.ui.getOcid()
                    self.ui.func_queue.append(self.ui.showChat)
                
                elif data[KDATA]["state"] == 1:
                    self.ui.func_queue.append(lambda:self.ui.setLoginTip(self.ui.lang[20]))
                    
    def getConfig(self):
        with open("./config/config.json", "r") as f:
            config_data = json.load(f)
        return config_data

    def setConfig(self, config_data):
        with open("./config/config.json", "w") as f:
            json.dump(config_data, f)

    def login(self, ocid, password):
        hash = hashlib.sha256()
        hash.update(password.encode(ENCODE))
        data = {"code": 6, "time":int(time.time()), "data": {"ocId": ocid, "password": hash.hexdigest()}}
        json_data = json.dumps(data)

        if self.connect_to_server:
            self.ocid = ocid
            self.sendByte(json_data.encode(ENCODE))
        else:
            self.ui.func_queue.append(lambda:self.ui.setLoginTip(self.ui.lang[3]))
    
    def register(self,nick,password):
        hash = hashlib.sha256()
        hash.update(password.encode(ENCODE))
        data = {"code" : 4,"time":int(time.time()),"data" : {"nick" : nick ,"password" : hash.hexdigest()}}
        json_data = json.dumps(data)

        if self.connect_to_server:
            self.sendByte(json_data.encode(ENCODE))
        else:
            self.ui.func_queue.append(lambda:self.ui.setLoginTip(self.ui.lang[3]))
    
    def sendNormalMsg(self,group,msg):
        if self.connect_to_server:
            data = {"code" : 0, "time":int(time.time()),"data" : {"cid" : group,"sender_id" : self.ocid,"msg" :msg}}
            json_data = json.dumps(data)
            self.sendByte(json_data.encode(ENCODE))
    
    def readChatPartRecord(self,chatId,num=-1,last_record_message_id=-1):
        data = self.getConfig()
        try:
            db = sqlite3.connect(f'{data[KDB]["folder"]}/{self.ocid}/{data[KDB]["record"]}')
            if last_record_message_id == -1:
                last_record_cursor = db.execute(f"SELECT * from chat{chatId} where (SELECT MAX(message_id) from chat{chatId})")
                for record in last_record_cursor:
                    last_record_message_id = record[0]
            record_list = []
            first_record_message_id = last_record_message_id-num
            if num == -1:
                first_record_message_id = 0

            records = db.execute(f"SELECT * from chat{chatId} where message_id >= {first_record_message_id} AND message_id <= {last_record_message_id}")
            for record in records:
                record_list.append(record)
            db.close()
            return record_list

        except sqlite3.OperationalError as e:
            if "no such table" in e.__str__():
                command = f'''
                CREATE TABLE "{chatId}" (
                    "message_id"	INTEGER UNIQUE,
                    "time"	INTEGER,
                    "msg_code"	INTEGER NOT NULL,
                    "msg_data"	TEXT,
                    PRIMARY KEY("message_id")
                );'''
                db.execute(command)
                return []
            print(e)
            db.close()
    
    def getChatsInfos(self):
        datas = []
        data = self.getConfig()
        db = sqlite3.connect(f'{data[KDB]["folder"]}/{self.ocid}/{data[KDB]["ourchat"]}')
        try:
            callback = db.execute("SELECT * from chats")
            for data in callback:
                datas.append(data)
            return datas
        except sqlite3.OperationalError as e:
            if "no such table" in e.__str__():
                command = '''
                CREATE TABLE "chats" (
                    "cid"	INTEGER NOT NULL UNIQUE,
                    PRIMARY KEY("cid")
                );'''
                db.execute(command)
                return []
    
    def addChatToChatInfos(self,cid):
        data = self.getConfig()
        db = sqlite3.connect(f'{data[KDB]["folder"]}/{self.ocid}/{data[KDB]["ourchat"]}')
        command = f"INSERT INTO chats VALUES({cid});"
        try:
            db.execute(command)
            db.commit()
            db.close()
        except sqlite3.OperationalError as e:
            if "no such table" in e.__str__():
                command = '''
                CREATE TABLE "chats" (
                    "cid"	INTEGER NOT NULL UNIQUE,
                    PRIMARY KEY("cid")
                );'''
                db.execute(command)
                db.close()
                self.addChatToChatInfos(cid)

    def addRecord(self,msg_data):
        data = self.getConfig()
        try:
            db = sqlite3.connect(f'{data[KDB]["folder"]}/{self.ocid}/{data[KDB]["record"]}')
        except sqlite3.OperationalError as e:
            if "unable to open database file" in e.__str__():
                os.chdir(data[KDB]["folder"])
                os.mkdir(self.ocid)
                os.chdir("..")
            self.addRecord(msg_data)
            return None

        try:
            cursor = db.execute(f'SELECT max(message_id) FROM "chat{msg_data[KDATA]["cid"]}";')
            for message_id in cursor:
                if message_id[0] == None:
                    last_message_id = 0
                else:
                    last_message_id = message_id[0]+1
    
            db.execute(f'INSERT INTO "chat{msg_data[KDATA]["cid"]}" VALUES({last_message_id},{int(time.time())},{msg_data[KCODE]},"{msg_data}");')
            db.commit()
            
            db.close()
        except sqlite3.OperationalError as e:
            if "no such table" in e.__str__():
                self.addChatToChatInfos(msg_data[KDATA]["cid"])
                command = f'''
                    CREATE TABLE "chat{msg_data[KDATA]["cid"]}" (
                    "message_id"	INTEGER UNIQUE,
                    "time"	INTEGER,
                    "msg_code"	INTEGER NOT NULL,
                    "msg_data"	TEXT,
                    PRIMARY KEY("message_id")
                );'''
                db.execute(command)
                db.close()
                self.addRecord(msg_data)
            else:
                print(e)
        