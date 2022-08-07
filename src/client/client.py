from config.const import *
from threading import Thread
import socket,json

class Client(Thread):
    def __init__(self):
        self.ip = self.getIp()
        self.server_ip = ""
        self.s = None
        self.ui = None
        super().__init__()
    
    def getIp(self):
        return socket.gethostbyname(socket.gethostname())

    def setServerIp(self,ip):
        self.server_ip = ip

    def setUi(self,ui):
        self.ui = ui

    def run(self):
        self.connectToHost()
    
    def connectToHost(self):
        while True:
            print(f"连接到服务器({self.server_ip})...")
            try:
                self.s = socket.socket()
                self.s.connect((self.server_ip,SERVERPORT))
                self.StartListen()
            except:
                print("连接失败 重新尝试连接")
                if self.ui != None and hasattr(self.ui,"CantConnectServer"):
                    self.ui.CantConnectServer.show()

    def sendByte(self,byte):
        self.s.send(byte)

    def sendNormorMessage(self,string,group):
        data = {KMSGCODE:NORMALCODE,KMSGDATA:{KDATA:string,KGROUP:group}}
        json_data = json.dumps(data)
        self.sendByte(json_data.encode(ENCODE))
    
    def StartListen(self):
        if self.ui != None and hasattr(self.ui,"CantConnectServer"):
            self.ui.CantConnectServer.hide()
        
        print("connect to server")
        while True:
            recv = self.s.recv(RECVMAX)
            if recv.decode(ENCODE) == "":
                continue
            print(recv.decode(ENCODE))
