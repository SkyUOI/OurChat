import os
import shutil


def remove_dir(path):
    os.chdir(path)
    for i in os.listdir():
        if os.path.isdir(i):
            remove_dir(i)
        else:
            os.remove(i)
    os.chdir("..")
    os.rmdir(path)


os.chdir("client/pc/src")
if "out" in os.listdir():
    remove_dir("out")
print("-" * 45, "Start to build", "-" * 45)
os.system(
    "python -m nuitka --standalone --output-dir=out --show-memory --enable-plugin=pyqt6 --remove-output --windows-console-mode=disable --windows-icon-from-ico=resources/images/logo.ico --macos-app-icon=resources/images/logo.ico --linux-icon=resources/images/logo.ico --plugin-enable=upx main.py"
)
print("-" * 45, "Over", "-" * 45)
print("copy resources...")
shutil.copytree("resources", "out/main.dist/resources")
print("copy languages...")
shutil.copytree("lang", "out/main.dist/lang")
os.chdir("../../..")
print("export themes...")
os.system("python script/export_themes.py --dir=client/pc/src/out/main.dist")
print("Please check /client/pc/src/out/main.dist")
