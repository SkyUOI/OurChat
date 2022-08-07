import client,uiSystem,sys
from PyQt5.QtWidgets import QApplication,QMainWindow
clientSystem = client.Client()
clientSystem.setDaemon(True)

ui = uiSystem.UiCotrol(clientSystem)
