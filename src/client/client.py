from config.const import *
from threading import Thread

import socket, json,hashlib


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
                    self.ui.CantConnectServer.show()

    def sendByte(self, byte):
        if self.connect_to_server:
            self.s.send(byte)
        else:
            return False

    def startListen(self):
        if self.ui != None:
            self.ui.CantConnectServer.hide()
        self.connect_to_server = True
        print("connect to server")
        while True:
            recv = self.s.recv(RECVMAX)
            if recv.decode(ENCODE) == "":
                continue
            json_data = recv.decode(ENCODE)
            data = json.loads(json_data)
            
            if data[KCODE] == NORMALCODE:
                print("文本信息",data[KDATA])
            
            elif data[KCODE] == REGISTERRETURNCODE:
                if data[KDATA]["state"] == 0:
                    self.ocid = data[KDATA]["ocId"]
                    self.ui.showChat()

                elif data[KDATA]["state"] == 1:
                    self.ui.setLoginTip(self.ui.lang[19])
            
            elif data[KCODE] == LOGINRETURNCODE:
                if data[KDATA]["state"] == 0:
                    self.ocid = self.ui.getOcid()
                    self.ui.showChat()
                
                elif data[KDATA]["state"] == 1:
                    self.ui.setLoginTip(self.ui.lang[20])
                    


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
        data = {"code": 6, "data": {"ocId": ocid, "password": hash.hexdigest()}}
        json_data = json.dumps(data)

        if self.connect_to_server:
            self.ocid = ocid
            self.sendByte(json_data.encode(ENCODE))
        else:
            self.ui.setLoginTip(self.ui.lang[3])
    
    def register(self,nick,password):
        hash = hashlib.sha256()
        hash.update(password.encode(ENCODE))
        data = {"code" : 4,"data" : {"nick" : nick ,"password" : hash.hexdigest()}}
        json_data = json.dumps(data)

        if self.connect_to_server:
            self.sendByte(json_data.encode(ENCODE))
        else:
            self.ui.setLoginTip(self.ui.lang[3])
    
    def sendNormalMsg(self,group,msg):
        if self.connect_to_server:
            data = {"code" : 0,"data" : {"cid" : group,"sender_id" : self.ocid,"msg" :msg}}
            json_data = json.dumps(data)
            self.sendByte(json_data.encode(ENCODE))
