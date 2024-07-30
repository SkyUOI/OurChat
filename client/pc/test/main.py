from websockets.server import serve
import asyncio
import os

samples = os.listdir("message_samples")


async def func(a):
    message = await a.recv()
    print(f">>> {message}")
    print("-" * 50)
    for i in range(len(samples)):
        print(f"{i+1}. {samples[i]}")
    print("-" * 50)
    index = int(input()) - 1
    with open(f"message_samples/{samples[index]}", "r") as f:
        await a.send(f.read())


async def main():
    async with serve(func, "127.0.0.1", 7777):
        await asyncio.Future()


asyncio.run(main())
