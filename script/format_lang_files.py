#!/usr/bin/env python3

import os

os.chdir("client/pc/src/lang")
for lang in os.listdir():
    translate = {}
    with open(lang, "r", encoding="utf-8") as f:
        for line in f.readlines():
            line = line.strip()
            line = line.split("#")[0]
            if "=" not in line:
                continue
            key, value = line.split("=")
            key, value = key.strip(), value.strip()
            translate[key] = value
    keys = list(translate.keys())
    keys.sort()
    with open(lang, "w", encoding="utf-8") as f:
        for key in keys:
            f.write(f"{key}={translate[key]}\n")
