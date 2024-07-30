from websockets.server import serve
from websockets.server import WebSocketServerProtocol
import asyncio
import os

samples = os.listdir("message_samples")


async def func(a: WebSocketServerProtocol):
    print(a.remote_address)
    while True:
        message = await a.recv()
        print(f"{a.remote_address} >>> {message}")
        await a.send(message)
    # print("-" * 50)
    # for i in range(len(samples)):
    #     print(f"{i+1}. {samples[i]}")
    # print("-" * 50)
    # index = int(input()) - 1
    # with open(f"message_samples/{samples[index]}", "r") as f:
    #     data = f.read()
    # await a.send(data)


async def main():
    async with serve(func, "127.0.0.1", 7777):
        await asyncio.Future()


asyncio.run(main())
