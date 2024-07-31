from websockets.sync import client
from websockets.exceptions import ConnectionClosedError as CloseError
from websockets.exceptions import ConnectionClosedOK as CloseOK
import json


class Connection:
    def __init__(self, ourchat):
        self.ourchat = ourchat
        self.conn = None
        self.setServer("127.0.0.1", 7777)

    def setServer(self, ip, port):
        self.ip = ip
        self.port = port

    def connect(self):
        if self.conn is not None:
            self.conn.close()
            self.conn = None
        try:
            self.conn = client.connect(f"ws://{self.ip}:{self.port}")
            return True, None
        except Exception as e:
            self.conn = None
            return False, e

    def send(self, data):
        json_str = json.dumps(data)
        self.conn.send(json_str)

    def recv(self):
        while True:
            try:
                message = self.conn.recv()
                data = json.loads(message)
                self.ourchat.getMessage(data)
            except CloseError:
                print("closeerror")
                flag = False
                times = 0
                while not flag or times < 5:
                    flag = self.connect()
                    times += 1
                    print("reconnect...")
                if not flag:
                    continue
            except CloseOK:
                return
            except Exception:
                print(Exception)

    def close(self):
        if self.conn is None:
            return
        self.conn.close()
        self.conn = None
