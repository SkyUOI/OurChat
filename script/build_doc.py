import os,shutil

os.chdir("..")
shutil.rmtree("docs/html")
os.mkdir("docs/html")
os.system("doxygen")
input("ok")
