import asyncio
import hashlib
import json
import os
import random
import sys
import time
from threading import Thread

import flask
from flask import make_response, request
from peewee import IntegerField, Model, SqliteDatabase, TextField, fn
from websockets.server import serve

app = flask.Flask(__name__)

db = SqliteDatabase("database.db")

upload_queue = {}


@app.route("/upload", methods=["POST"])
def upload():
    global upload_queue
    key = request.headers["Key"]
    if key not in upload_queue:
        return make_response("<h1>404</h1>", 404)
    hash = upload_queue[key]
    data = request.get_data()
    update_hash = hashlib.sha256()
    update_hash.update(data)
    print(hash, update_hash.hexdigest())
    if hash == update_hash.hexdigest():
        with open(f"./files/{key}", "wb") as f:
            f.write(data)
        upload_queue.pop(key)
        return make_response("<h1>200</h1>", 200)
    return make_response("<h1>400</h1>", 400)


@app.route("/download", methods=["POST"])
def download():
    key = request.headers["Key"]
    if key in os.listdir("files"):
        with open(f"./files/{key}", "rb") as f:
            return f.read()
    return make_response(None, 404)


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
    name = TextField(null=True)
    avatar = TextField(null=True)
    avatar_hash = TextField(null=True)
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
                sample, func, need_await = auto_reply[data["code"]]
                with open(f"message_samples/{sample}", "r") as f:
                    sample_data = json.loads(f.read())
                if need_await:
                    reply_data = await func(self, sample_data, data)
                else:
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
    account_info = Account.get_or_none(Account.ocid == data["ocid"])
    if account_info is not None:
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
        sample["status"] = 0
    else:
        sample["status"] = 3
    return sample


def login(conn: Connection, sample: dict, data: dict) -> dict:
    if data["login_type"] == 0:
        email = data["account"]
        account = Account.get_or_none(Account.email == email)
    else:
        ocid = data["account"]
        account = Account.get_or_none(Account.ocid == ocid)
    if account is None:
        sample["status"] = 5
    else:
        if data["password"] == account.password:
            conn.ocid = account.ocid
            sample["ocid"] = account.ocid
            sample["status"] = 0
        else:
            sample["status"] = 5

    return sample


def session_info(conn: Connection, sample: dict, data: dict) -> dict:
    session = Session.get_or_none(Session.session_id == data["session_id"])
    if session is None:
        sample["status"] = 3
    else:
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
        sample["status"] = 0
    return sample


def register(conn: Connection, sample: dict, data: dict) -> dict:
    account = Account.get_or_none(Account.email == data["email"])
    if account is not None:
        sample["status"] = 4
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
            sessions=json.dumps([]),
            friends=json.dumps([]),
        )
        conn.ocid = ocid
        sample["ocid"] = ocid
        sample["status"] = 0

    return sample


def normal(conn: Connection, sample: dict, data: dict) -> dict:
    return sample


async def user_msg(conn: Connection, sample: dict, data: dict) -> dict:
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
        if user_conn.ocid in json.loads(session.members):
            await user_conn.send(sample)

    return sample


async def new_session(conn: Connection, sample: dict, data: dict) -> dict:
    avatar = None
    avatar_hash = None
    name = None

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
        session_id = "0" * (10 - len(str(int(max_session_id)))) + str(
            int(max_session_id) + 1
        )

    for ocid in data["members"]:
        account = Account.get(Account.ocid == ocid)
        sessions = json.loads(account.sessions)
        sessions.append(session_id)
        Account.update(sessions=json.dumps(sessions)).where(
            Account.ocid == ocid
        ).execute()

    Session.create(
        session_id=session_id,
        name=name,
        avatar=avatar,
        avatar_hash=avatar_hash,
        time=time.time(),
        update_time=time.time(),
        members=json.dumps(data["members"]),
        owner=conn.ocid,
    )

    sample["status"] = 0
    sample["session_id"] = session_id

    for address in connections:
        user_conn = connections[address]
        if user_conn == conn:
            continue
        if user_conn.ocid in data["members"]:
            await user_conn.send(sample)

    return sample


def unregister(conn: Connection, sample: dict, data: dict) -> dict:
    account = Account.get_or_none(Account.ocid == conn.ocid)
    if account is None:
        return sample
    account.delete_instance()
    sample["status"] = 0
    return sample


def set_account(conn: Connection, sample: dict, data: dict) -> dict:
    account = Account.get_or_none(Account.ocid == conn.ocid)
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


def upload_msg(conn: Connection, sample: dict, data: dict) -> dict:
    global upload_queue
    key = "".join([str(random.randint(0, 9)) for _ in range(15)])
    upload_queue[key] = data["hash"]
    sample["key"] = key
    sample["status"] = 0
    sample["hash"] = data["hash"]
    return sample


auto_reply = {
    0: ("user_msg.json", user_msg, True),
    1: ("session_info.json", session_info, False),
    4: ("register.json", register, False),
    6: ("login.json", login, False),
    8: ("new_session.json", new_session, True),
    10: ("account_info.json", account_info, False),
    12: ("server_status.json", normal, False),
    14: ("verify_status.json", normal, False),
    16: ("unregister.json", unregister, False),
    19: ("set_account.json", set_account, False),
    21: ("upload.json", upload_msg, False),
}


async def connected(websocket):
    await Connection(websocket).recvAndReply()


async def server_forever(ip, port):
    async with serve(connected, ip, port):
        await asyncio.Future()


def start_serve(ip, port):
    asyncio.run(server_forever(ip, port))


def start_http(ip, port):
    app.run(host=ip, port=port)


async def main():
    ip = "localhost"
    port = 7777
    if "--ip" in sys.argv:
        ip = sys.argv[sys.argv.index("--ip") + 1]
    if "--port" in sys.argv:
        port = int(sys.argv[sys.argv.index("--port") + 1])

    db.connect()
    Account.create_table(safe=True)
    Session.create_table(safe=True)

    if "files" not in os.listdir():
        os.mkdir("files")

    Thread(target=start_serve, daemon=True, args=(ip, port)).start()
    start_http(ip, port + 1)

    # while True:
    #     print("=" * 40)
    #     for i in range(len(messages)):
    #         print(f"{i+1}. {messages[i]}")
    #     print("=" * 40)
    #     user_input = input(
    #         "input {address}<<<{msgid}/{json} to send message\ninput blank to exit\n"
    #     )
    #     if user_input == "":
    #         break
    #     address, data = user_input.split("<<<")
    #     send_data = None
    #     try:
    #         index = int(data)
    #         with open(f"message_samples/{messages[index-1]}") as f:
    #             send_data = json.loads(f.read())
    #     except ValueError:
    #         send_data = json.loads(data)
    #     if address not in connections:
    #         print("[ERROR] address not found")
    #         continue
    #     await connections[address].send(send_data)

    print("CLOSING...")
    keys = list(connections.keys())
    for address in keys:
        connections[address].close()
    db.close()


if __name__ == "__main__":
    asyncio.run(main())
