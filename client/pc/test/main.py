import json
import os
import random
import time
from threading import Thread

from websockets import ConnectionClosedOK
from websockets.sync.server import serve

samples = list(os.listdir("message_samples"))
code = None
recommend_answer = {
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


def recv(s):
    global code
    while True:
        try:
            msg = s.recv()
        except ConnectionClosedOK:
            break
        print(">>>", msg)
        data = json.loads(msg)
        code = data["code"]
        print("=" * 50)
        for i in range(len(samples)):
            recommend = " "
            if samples[i] in recommend_answer[data["code"]]:
                recommend = "@"
            print(f"[{recommend}]{i+1}. {samples[i]}")
        print("=" * 50)


def server(c, s):
    global code
    Thread(target=recv, args=(c,), daemon=True).start()
    print("Connected!")
    while True:
        answer = input("")
        if answer == "exit":
            c.close()
            break
        try:
            if answer != "":
                index = int(answer)
                if index == 0:
                    answer = json.dumps(
                        {
                            "code": 0,
                            "time": int(time.time()),
                            "msg_id": str(random.randint(-10000000000, 1000000000)),
                            "sender": {"ocid": "0000000000", "session_id": "114514"},
                            "msg": [{"type": 0, "text": f"It's {time.time()} now."}],
                        }
                    )
                else:
                    with open(f"message_samples/{samples[index-1]}", "r") as f:
                        answer = f.read()
        except Exception as e:
            print(e)
        if answer == "":
            with open(f"message_samples/{recommend_answer[code][0]}", "r") as f:
                print(f"USE {recommend_answer[code][0]}")
                answer = f.read()
        c.send(answer)
    c.close()
    s.shutdown()


print("Waiting for connection...")
with serve(lambda c: server(c, s), host="127.0.0.1", port=7777) as s:
    s.serve_forever()
