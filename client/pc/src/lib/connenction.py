from websockets.sync import client
from websockets.exceptions import ConnectionClosedError as CloseError
from websockets.exceptions import ConnectionClosedOK as CloseOK
import json


class Connection:
    def __init__(self, ip="127.0.0.1", port=7777):
        self.conn = None
        self.setServer(ip, port)

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
        except Exception:
            self.conn = None
            return False, Exception

    def send(self, data):
        json_str = json.dumps(data)
        self.conn.send(json_str)

    def recv(self):
        while True:
            try:
                message = self.conn.recv()
                print(message)
            except CloseError:
                flag = False
                times = 0
                while not flag or times < 5:
                    flag = self.connect()
                    times += 1
                    print("reconnect...")
                if not flag:
                    continue
            except CloseOK:
                print("closeok")
                return

    def close(self):
        if self.conn is None:
            return
        self.conn.close()
        self.conn = None
