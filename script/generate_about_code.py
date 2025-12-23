import requests, time, hashlib, json, os, argparse


def get_github_contributors():
    url = "https://api.github.com/repos/skyuoi/ourchat/contributors"
    response = requests.get(url)
    if response.status_code == 200:
        raw_contributors_list = response.json()
        contributors_list = []
        for contributor in raw_contributors_list:
            if contributor["type"] == "User":
                contributors_list.append(
                    {
                        "user": contributor["login"],
                        "avatar": contributor["avatar_url"],
                        "url": contributor["html_url"],
                    }
                )
        return contributors_list
    else:
        print(f"Failed to fetch contributors: {response.status_code}")
        return []


def get_afdian_donors(USERID, TOKEN, total_pages=-1):
    i = 0
    donors = {}
    while total_pages == -1 or i < total_pages:
        raw_data = {
            "user_id": USERID,
            "page": 1,
            "ts": str(int(time.time())),
            "params": json.dumps({"per_page": 100, "page": 1}),
        }
        raw_sign = TOKEN
        for key in sorted(raw_data.keys()):
            raw_sign += key + str(raw_data[key])
        data = raw_data
        data["sign"] = hashlib.md5(raw_sign.encode("utf-8")).hexdigest().upper()
        raw_data = requests.get(
            url="https://afdian.com/api/open/query-sponsor", params=data
        ).json()
        total_pages = raw_data["data"]["total_count"]
        i += 1
        for j in raw_data["data"]["list"]:
            if j["user"]["user_id"] not in donors.keys():
                donors[j["user"]["user_id"]] = {
                    "user": j["user"]["name"],
                    "avatar": j["user"]["avatar"],
                    "amount": 0,
                }
            donors[j["user"]["user_id"]]["amount"] += float(j["all_sum_amount"])
    sorted_donors = sorted(
        [donors[key] for key in donors], key=lambda x: x["amount"], reverse=True
    )
    for i in range(len(sorted_donors)):
        sorted_donors[i].pop("amount")
    return sorted_donors


def splitCode():
    with open("./client/lib/core/const.dart", "r", encoding="utf-8") as f:
        code = f.readlines()
    return (
        code[: code.index("// ===== AUTO GENERATED CODE BEGIN =====\n") + 1],
        code[code.index("// ===== AUTO GENERATED CODE END =====\n") :],
    )


argparser = argparse.ArgumentParser()
argparser.add_argument("--afdian_userid", type=str)
argparser.add_argument("--afdian_token", type=str)
argparser.add_argument("--version", type=str)
argparser.add_argument("--commit_sha", type=str)
args = argparser.parse_args()

code = splitCode()

with open("./client/lib/core/const.dart", "w", encoding="utf-8") as f:
    for i in code[0]:
        f.write(i)
    print(
        "write "
        + f"const List<Map<String, String>> contributorsList = {json.dumps(get_github_contributors(), ensure_ascii=False)};"
    )
    f.write(
        f"const List<Map<String, String>> contributorsList = {json.dumps(get_github_contributors(), ensure_ascii=False)};\n"
    )
    if args.afdian_userid and args.afdian_token:
        print(
            "write "
            + f"const List<Map<String, String>> donorsList = {json.dumps(get_afdian_donors(USERID=args.afdian_userid, TOKEN=args.afdian_token), ensure_ascii=False)};"
        )
        f.write(
            f"const List<Map<String, String>> donorsList = {json.dumps(get_afdian_donors(USERID=args.afdian_userid, TOKEN=args.afdian_token), ensure_ascii=False)};\n"
        )
    else:
        print("write const List<Map<String, String>> donorsList = [];")
        f.write("const List<Map<String, String>> donorsList = [];\n")
    print("write " + f'const currentVersion = "{args.version}";')
    f.write(f'const currentVersion = "{args.version}";\n')
    print("write " + f'const currentCommitSha = "{args.commit_sha}";')
    f.write(f'const currentCommitSha = "{args.commit_sha}";\n')
    for i in code[1]:
        f.write(i)


os.system("dart format ./client/lib/core/const.dart")
