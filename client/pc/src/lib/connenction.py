import websockets
import json


class Connection:
    def __init__(self, server_ip="127.0.0.1", port=7777):
        self.server_ip = server_ip
        self.port = port

    async def connect(self):
        try:
            self.conn = await websockets.connect(f"ws://{self.server_ip}:{self.port}")
            return True
        except Exception as e:
            print(e)
            return False

    async def send(self, data):
        json_str = json.dumps(data)
        await self.conn.send(json_str)
        response_json = await self.conn.recv()
        response_data = json.loads(response_json)
        return response_data

    async def close(self):
        await self.conn.close()

    async def ping(self):
        await self.conn.ping()
