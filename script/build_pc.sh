#!/bin/sh

cd client/pc/src
python -m nuitka --standalone --output-dir=out --show-memory --enable-plugin=pyqt5 --remove-output --windows-disable-console main.py
pause