import os

os.chdir("..")
os.system("git checkout main")
os.system("git merge dev")
os.system("git push")
os.system("git checkout dev")
input("ok")
