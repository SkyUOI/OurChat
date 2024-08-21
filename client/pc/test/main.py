import asyncio
import json
import os
import random
import time
from threading import Thread

from peewee import IntegerField, Model, SqliteDatabase, TextField, fn
from websockets.server import serve

db = SqliteDatabase("database.db")
db.connect()


class Account(Model):
    ocid = TextField(null=False, primary_key=True)
    email = TextField(null=False)
    password = TextField(null=False)
    nickname = TextField(null=False)
    status = IntegerField(null=False)
    avatar = TextField(null=False)
    avatar_hash = TextField(null=False)
    time = IntegerField(null=False)
    public_update_time = IntegerField(null=False)
    update_time = IntegerField(null=False)
    sessions = TextField(null=False)
    friends = TextField(null=False)

    class Meta:
        database = db
        table_name = "account"


class Session(Model):
    session_id = TextField(null=False, primary_key=True)
    name = TextField(null=False)
    avatar = TextField(null=False)
    avatar_hash = TextField(null=False)
    time = IntegerField(null=False)
    update_time = IntegerField(null=False)
    members = TextField(null=False)
    owner = TextField(null=False)

    class Meta:
        database = db
        table_name = "session"


messages = os.listdir("message_samples")
s = None
connections = {}


class Connection:
    def __init__(self, conn):
        global connections
        self.conn = conn
        self.address = f"{self.conn.remote_address[0]}:{self.conn.remote_address[1]}"
        self.ocid = None
        connections[self.address] = self
        print(f"[INFO]({self.address})>>> CONNECTED")

    async def send(self, data):
        await self.conn.send(json.dumps(data))
        print(f"[INFO]({self.address})<<< SEND MESSAGE {data}")

    async def recvAndReply(self):
        global connections
        async for message in self.conn:
            data = json.loads(message)
            print(f"[INFO]({self.address})>>> GET MESSAGE {data}")
            if data["code"] in auto_reply:
                sample, func = auto_reply[data["code"]]
                with open(f"message_samples/{sample}", "r") as f:
                    sample_data = json.loads(f.read())
                reply_data = func(self, sample_data, data)
                await self.conn.send(json.dumps(reply_data))
                print(f"[INFO]({self.address})>>> AUTO-REPLY MESSAGE {reply_data}")
        print(f"[INFO]({self.address})>>> CLOSE")
        connections.pop(self.address)

    def close(self):
        self.conn.close()
        connections.pop(self.address)
        print(f"[INFO]({self.address})<<< CLOSE")


def account_info(conn: Connection, sample: dict, data: dict) -> dict:
    print(len(data["ocid"]))
    account_info = Account.get_or_none(Account.ocid == data["ocid"])
    for key in data["request_values"]:
        if key == "ocid":
            sample["data"][key] = account_info.ocid
        elif key == "nickname":
            sample["data"][key] = account_info.nickname
        elif key == "status":
            sample["data"][key] = account_info.status
        elif key == "avatar":
            sample["data"][key] = account_info.avatar
        elif key == "avatar_hash":
            sample["data"][key] = account_info.avatar_hash
        elif key == "time":
            sample["data"][key] = account_info.time
        elif key == "public_update_time":
            sample["data"][key] = account_info.public_update_time
        elif key == "update_time":
            sample["data"][key] = account_info.update_time
        elif key == "sessions":
            sample["data"][key] = account_info.sessions
            if conn.ocid != account_info.ocid:
                sample["data"][key] = None
        elif key == "friends":
            sample["data"][key] = account_info.friends
            if conn.ocid != account_info.ocid:
                sample["data"][key] = None
    return sample


def login(conn: Connection, sample: dict, data: dict) -> dict:
    if data["login_type"] == 0:
        email = data["account"]
        account = Account.get_or_none(Account.email == email)
    else:
        ocid = data["account"]
        account = Account.get_or_none(Account.ocid == ocid)
    if account is None:
        sample["status"] = 1
    else:
        if data["password"] == account.password:
            conn.ocid = account.ocid
            sample["ocid"] = account.ocid
            sample["status"] = 0
        else:
            sample["status"] = 1

    return sample


def session_info(conn: Connection, sample: dict, data: dict) -> dict:
    session = Session.get_or_none(Session.session_id == data["session_id"])
    for key in data["request_values"]:
        if key == "session_id":
            sample["data"][key] = session.session_id
        elif key == "name":
            sample["data"][key] = session.name
        elif key == "avatar":
            sample["data"][key] = session.avatar
        elif key == "avatar_hash":
            sample["data"][key] = session.avatar_hash
        elif key == "time":
            sample["data"][key] = session.time
        elif key == "update_time":
            sample["data"][key] = session.update_time
        elif key == "members":
            sample["data"][key] = session.members
        elif key == "owner":
            sample["data"][key] = session.owner
    return sample


def register(conn: Connection, sample: dict, data: dict) -> dict:
    account = Account.get_or_none(Account.email == data["email"])
    if account is not None:
        sample["status"] = 2
    else:
        max_ocid = Account.select(fn.Max(Account.ocid)).scalar()
        if max_ocid is None:
            ocid = "0" * 10
        else:
            ocid = "0" * (10 - len(str(int(max_ocid)))) + str(int(max_ocid) + 1)
        Account.create(
            ocid=ocid,
            email=data["email"],
            password=data["password"],
            nickname="OurChat User",
            status=0,
            avatar="http://img.senlinjun.top/imgs/2024/08/c57c426151947784.png",
            avatar_hash="6856e25c44cce62e5577c23506bcfea8fdd440ad63594ef82a5d9e36951e240a",
            time=time.time(),
            public_update_time=time.time(),
            update_time=time.time(),
            sessions=[],
            friends=[],
        )
        conn.ocid = ocid
        sample["ocid"] = ocid
        sample["status"] = 0

    return sample


def normal(conn: Connection, sample: dict, data: dict) -> dict:
    return sample


def user_msg(conn: Connection, sample: dict, data: dict) -> dict:
    session = Session.get_or_none(Session.session_id == data["sender"]["session_id"])
    if session is None:
        return
    sample["time"] = time.time()
    sample["msg_id"] = "".join([str(random.randint(0, 9)) for _ in range(10)])
    sample["sender"]["ocid"] = conn.ocid
    sample["sender"]["session_id"] = data["sender"]["session_id"]
    sample["msg"] = data["msg"]

    for address in connections:
        user_conn = connections[address]
        if user_conn == conn:
            continue
        if user_conn in session.members:
            user_conn.send(sample)

    return sample


def new_session(conn: Connection, sample: dict, data: dict) -> dict:
    avatar = "http://img.senlinjun.top/imgs/2024/08/c57c426151947784.png"
    avatar_hash = "6856e25c44cce62e5577c23506bcfea8fdd440ad63594ef82a5d9e36951e240a"
    name = "%OURCHAT_DEFAULT_SESSION_NAME%"

    if "avatar" in data:
        avatar = data["avatar"]
    if "avatar_hash" in data:
        avatar_hash = data["avatar_hash"]
    if "name" in data:
        name = data["name"]

    max_session_id = Session.select(fn.Max(Session.session_id)).scalar()
    if max_session_id is None:
        session_id = "0" * 10
    else:
        session_id = "0" * (10 - str(int(max_session_id) + 1)) + str(
            int(max_session_id) + 1
        )

    Session.create(
        session_id=session_id,
        name=name,
        avatar=avatar,
        avatar_hash=avatar_hash,
        time=time.time(),
        update_time=time.time(),
        members=data["members"],
        owner=conn.ocid,
    )

    sample["status"] = 0
    sample["session_id"] = session_id

    return sample


def unregister(conn: Connection, sample: dict, data: dict) -> dict:
    account = Account.get_or_none(Account.ocid == conn.ocid)
    if account is None:
        return sample
    account.delete_instance()
    sample["status"] = 0
    return sample


def set_account(conn: Connection, sample: dict, data: dict) -> dict:
    if conn.ocid != data["ocid"]:
        sample["status"] = 1
        return sample
    account = Account.get_or_none(Account.ocid == data["ocid"])
    nickname = account.nickname
    status = account.status
    avatar = account.avatar
    avatar_hash = account.avatar_hash
    public_update_time = account.public_update_time
    for key in data["data"]:
        if key == "nickname":
            nickname = data["data"][key]
        elif key == "status":
            status = data["data"][key]
        elif key == "avatar":
            avatar = data["data"][key]
        elif key == "avatar_hash":
            avatar_hash = data["data"][key]
    public_update_time = time.time()
    Account.update(
        nickname=nickname,
        status=status,
        avatar=avatar,
        avatar_hash=avatar_hash,
        public_update_time=public_update_time,
    ).where(Account.ocid == data["ocid"]).execute()
    sample["status"] = 0
    return sample


auto_reply = {
    0: ("user_msg.json", user_msg),
    1: ("session_info.json", session_info),
    4: ("register.json", register),
    6: ("login.json", login),
    10: ("account_info.json", account_info),
    12: ("server_status.json", normal),
    14: ("verify_status.json", normal),
    16: ("unregister.json", unregister),
    19: ("set_account.json", set_account),
}


async def connected(websocket):
    await Connection(websocket).recvAndReply()


async def server_forever():
    async with serve(connected, "localhost", 7777):
        await asyncio.Future()  # run forever


def start_serve():
    asyncio.run(server_forever())


Account.create_table(safe=True)
Session.create_table(safe=True)

Thread(target=start_serve, daemon=True).start()

while True:
    print("=" * 40)
    for i in range(len(messages)):
        print(f"{i+1}. {messages[i]}")
    print("=" * 40)
    user_input = input(
        "input {address}<<<{msgid}/{json} to send message\ninput blank to exit\n"
    )
    if user_input == "":
        break
    address, data = user_input.split("<<<")
    send_data = None
    try:
        index = int(data)
        with open(f"message_samples/{messages[index-1]}") as f:
            send_data = json.loads(f.read())
    except ValueError:
        send_data = json.loads(data)
    if address not in connections:
        print("[ERROR] address not found")
        continue
    connections[address].send(send_data)

print("CLOSING...")
keys = list(connections.keys())
for address in keys:
    connections[address].close()
db.close()
