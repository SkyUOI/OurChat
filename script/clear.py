import os

os.chdir("../bin")

for i in os.listdir("."):
    try:
        os.remove(i)
    except:
        print(f"can't remove directory '{i}'")
    else:
        print("remove", i)

input("ok")