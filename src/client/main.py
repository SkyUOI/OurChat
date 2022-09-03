import sys, os, json

if "config" not in os.listdir():
    os.mkdir("config")
    const_py = """# socket\nSERVERPORT = 54088\nRECVMAX = 1024\nLISTENMAX = 1\nENCODE = "utf-8"\n\n# msgcode\nNORMALCODE = 0\nEMOJICODE = 1\nIMAGECODE = 2\nFILECODE = 3\nREGISTERCODE = 4\nREGISTERRETURNCODE = 5\nLOGINCODE = 6\nLOGINRETURNCODE = 7\n\n# filePath\nLowPixelLogoPath = "../../resource/OurChat_Logo_low_pixel.png"\nLogoPath = "../../resource/OurChat_Logo.png"\n\n# dataKey\nKMSGCODE = "msgcode"\nKMSGDATA = "msgdata"\nKDATA = "data"\nKGROUP = "group"\n"""
    config_json = {"server_ip": "127.0.0.1", "lang_file": "en-us.lang","database":{"record":"./database/record.db","ourchat":"./database/ourchat.db"},"show_record_num":50}
    with open("./config/const.py", "w") as f:
        f.write(const_py)
    with open("./config/config.json", "w") as f:
        json.dump(config_json, f, indent=1)

if "database" not in os.listdir():
    os.mkdir("database")

import client, uiSystem

clientSystem = client.Client()
clientSystem.setDaemon(True)

ui = uiSystem.UiCotrol(clientSystem)
