from websockets.sync.server import serve
from threading import Thread
import os
import json

samples = list(os.listdir("message_samples"))
recommend_answer = {
    4: ["reg_email_error.json", "reg_server_error.json", "reg_success.json"],
    6: [
        "login_server_error.json",
        "login_wrong_account_or_psw.json",
        "login_success.json",
    ],
    8: ["new_session_success.json", "new_session_error.json"],
    10: ["account_info.json"],
    12: ["server_maintenance.json", "server_status_normal.json"],
    14: [
        "verify_status_fail.json",
        "verify_status_success.json",
        "verify_status_timeout.json",
    ],
    16: ["sign_out_fail.json", "sign_out_success.json"],
}


def recv(s):
    while True:
        msg = s.recv()
        print(">>>", msg)
        data = json.loads(msg)
        print("=" * 50)
        for i in range(len(samples)):
            recommend = " "
            if samples[i] in recommend_answer[data["code"]]:
                recommend = "@"
            print(f"[{recommend}]{i+1}. {samples[i]}")
        print("=" * 50)


def server(s):
    Thread(target=recv, args=(s,), daemon=True).start()
    while True:
        answer = input("<<< ")
        if answer == "exit":
            s.close()
            break
        try:
            index = int(answer)
            with open(f"message_samples/{samples[index-1]}", "r") as f:
                answer = f.read()
        except Exception:
            pass
        s.send(answer)


with serve(server, host="127.0.0.1", port=7777) as s:
    s.serve_forever()
