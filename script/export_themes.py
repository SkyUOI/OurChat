#!/usr/bin/env python3

import os
import sys

import qt_material


def remove_dir(path):
    os.chdir(path)
    for i in os.listdir():
        if os.path.isdir(i):
            remove_dir(i)
        else:
            os.remove(i)
    os.chdir("..")
    os.rmdir(path)


if __name__ == "__main__":
    path = "./client/pc/src"
    for i in sys.argv[1:]:
        if "--dir=" in i:
            path = i.replace("--dir=", "")
            break
    os.chdir(path)
    if "theme" in os.listdir():
        remove_dir("theme")
    os.mkdir("theme")
    os.chdir("theme")

    for theme in qt_material.list_themes():
        name = theme.replace(".xml", "")
        os.mkdir(name)
        os.chdir(name)
        invert_secondary = False
        if "light" in name:
            invert_secondary = True
        qt_material.export_theme(
            theme,
            f"{name}.qss",
            "resources.rcc",
            invert_secondary=invert_secondary,
            output="resources",
            extra={
                "font_family": "{FONT_FAMILY}",
            },
        )
        with open(f"{name}.qss", "r") as f:
            qss = f.read()
        color = qss[
            qss.index("QPushButton {\n  color: ") + len("QPushButton {\n  color: ") :
        ].split(";")[0]
        with open(f"{name}.qss", "w") as f:
            f.write(f"/* COLOR: {color}; */\n")
            f.write(qss)
        os.chdir("..")
