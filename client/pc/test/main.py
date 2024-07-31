from websockets.server import serve
from websockets.server import WebSocketServerProtocol
import asyncio
import os
import json

samples = os.listdir("message_samples")
with open("auto_reply.json", "r") as f:
    auto_reply_data = json.load(f)


async def func(a: WebSocketServerProtocol):
    print(a.remote_address)
    while not a.closed:
        message = await a.recv()
        data = json.loads(message)
        index = auto_reply_data[data["code"]]
        print(index)
        # print(f"{a.remote_address} >>> {message}")
        # print("-" * 50)
        # for i in range(len(samples)):
        #     print(f"{i+1}. {samples[i]}")
        # print("-" * 50)
        # index = int(input()) - 1
        if index is not None:
            with open(f"message_samples/{index}", "r") as f:
                replay = f.read()
        else:
            replay = message
        print(f"<<< {replay}")
        await a.send(replay)


async def main():
    async with serve(func, "127.0.0.1", 7777):
        await asyncio.Future()


asyncio.run(main())
