import json
import os

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


def recv(c, s):
    global code
    ocid = None
    while True:
        try:
            msg = c.recv()
        except ConnectionClosedOK:
            break
        except Exception as e:
            print(e)
            break
        print(">>>", msg)
        data = json.loads(msg)
        code = data["code"]
        if code == 0:
            data["sender"]["ocid"] = ocid
        if code == 6:
            ocid = data["account"]
        if code == 4:
            ocid = "0000000000"
        # print("=" * 50)
        # for i in range(len(samples)):
        #     recommend = " "
        #     if samples[i] in recommend_answer[data["code"]]:
        #         recommend = "@"
        #     print(f"[{recommend}]{i+1}. {samples[i]}")
        # print("=" * 50)
        print(f"<<<{recommend_answer[data['code']][0]}")
        with open(
            f"message_samples/{recommend_answer[data['code']][0]}",
            "r",
            encoding="utf-8",
        ) as f:
            c.send(f.read())
    s.shutdown()


print("Waiting for connection...")
with serve(lambda c: recv(c, s), host="127.0.0.1", port=7777) as s:
    s.serve_forever()
