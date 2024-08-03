import os
import shutil

os.chdir("client/pc/src")
print("-" * 45, "Start to build", "-" * 45)
os.system(
    "python -m nuitka --standalone --output-dir=out --show-memory --enable-plugin=pyqt6 --remove-output --windows-console-mode=disable main.py"
)
print("-" * 45, "Over", "-" * 45)
print("copy resources...")
shutil.copytree("resources", "out/main.dist/resources")
os.chdir("../../..")
print("export themes...")
os.system("python script/export_themes.py --dir=client/pc/src/out/main.dist")
print("Please check /client/pc/src/out/main.dist")
