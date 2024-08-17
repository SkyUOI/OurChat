import json
import os
import random
from threading import Thread

from websockets import ConnectionClosedOK
from websockets.sync.server import serve

samples = list(os.listdir("message_samples"))
code = None
recommend_answer = {
    1: ["session_info.json"],
    4: ["reg_success.json", "reg_email_error.json", "reg_server_error.json"],
    6: [
        "login_success.json",
        "login_server_error.json",
        "login_wrong_account_or_psw.json",
    ],
    8: ["new_session_success.json", "new_session_error.json"],
    10: ["account_info.json"],
    12: ["server_status_normal.json", "server_maintenance.json"],
    14: [
        "verify_status_success.json",
        "verify_status_fail.json",
        "verify_status_timeout.json",
    ],
    16: ["sign_out_success.json", "sign_out_fail.json"],
}

account = {
    "0000000000": (
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "senlinjun",
        "http://img.senlinjun.top/imgs/2024/06/012bb970f4d8b5c3.jpg",
        "06c46dcc4d7c79cf89a5030e5cd6bd48826f97dc711c28d42507afdc1da7cbac",
        ["0000000001"],
    ),
    "0000000001": (
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "limuy",
        "http://img.senlinjun.top/imgs/2024/08/2d7b8b76596ae8bf.png",
        "04428d018bf89cd7271ffdd1c7271d0dd0071d1ad4a0664533c72defd0ab10cc",
        ["0000000000"],
    ),
}
sessions = {"114514": ["0000000000", "0000000001"]}
connections = {}


def main(conn, serve_obj):
    global connections
    ocid = None
    print("new connection")
    while True:
        try:
            message = conn.recv()
        except ConnectionClosedOK:
            break
        except Exception as e:
            print(e)
        data = json.loads(message)
        code = data["code"]
        response_data = None
        if code == 0:
            data["sender"]["ocid"] = ocid
            data["msg_id"] = "".join([str(random.randint(0, 9)) for _ in range(10)])
            for member in sessions[data["sender"]["session_id"]]:
                if member == ocid:
                    continue
                response_data = data
                if member not in connections:
                    continue
                connections[member].send(json.dumps(response_data))
                print(f"{member} >>> {json.dumps(response_data)}")

        elif code == 6:
            if (
                data["account"] in account
                and account[data["account"]][0] == data["password"]
            ):
                ocid = data["account"]
                response_data = {"code": 7, "time": 114514, "status": 0, "ocid": ocid}
                print(f"{ocid} login")
                connections[ocid] = conn
            else:
                response_data = "login_wrong_account_or_psw.json"

        elif code == 10:
            request_ocid = data["ocid"]
            password, nickname, avatar, avatar_hash, friends = account[request_ocid]
            response_data = {
                "code": 11,
                "time": 114514,
                "data": {
                    "ocid": request_ocid,
                    "nickname": nickname,
                    "status": "died",
                    "avatar": avatar,
                    "avatar_hash": avatar_hash,
                    "time": 114514,
                    "update_time": 17237184354380,
                    "sessions": ["114514"],
                    "friends": ["0000000001"],
                },
            }

        if response_data is None:
            response_data = recommend_answer[code][0]

        if isinstance(response_data, str):
            with open("message_samples/" + response_data, "r", encoding="utf-8") as f:
                response_data = json.loads(f.read())

        print(f"{ocid} >>> {json.dumps(response_data)}")
        conn.send(json.dumps(response_data))
    serve_obj.shutdown()


def serve_func():
    try:
        with serve(lambda conn: main(conn, serve_obj), "127.0.0.1", 7777) as serve_obj:
            serve_obj.serve_forever()
    except Exception as e:
        print(e)


while True:
    print("waiting for connect...")
    Thread(target=serve_func, daemon=True).start()
    _ = input('press enter to create a server\n"exit" to exit\n')
    if _ == "exit":
        for ocid in connections:
            connections[ocid].close()
        break
