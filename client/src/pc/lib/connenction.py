import websockets


class Connection:
    def __init__(self, server_ip="127.0.0.1", port=7777):
        self.server_ip = server_ip
        self.port = port

    def connect(self):
        self.conn = websockets.connect(f"ws://{self.server_ip}:{self.port}")
