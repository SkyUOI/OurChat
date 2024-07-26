from PyQt5.QtWidgets import QApplication,QMainWindow
from PyQt5.QtCore import QCoreApplication,Qt
from qt_material import apply_stylesheet,list_themes
from ui_logic.main import Ui_Main
import sys

class UISystem:
    def __init__(self,argv):
        QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
        QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)
        QApplication.setHighDpiScaleFactorRoundingPolicy(Qt.HighDpiScaleFactorRoundingPolicy.Round)
        self.app = QApplication(sys.argv)
        self.mainwindow = QMainWindow()
        self.ui = None
        self.dialogs = []
        self.theme = "dark_amber.xml"

        apply_stylesheet(self.app,self.theme)
    

    def setUI(self,ui):
        self.ui = ui(self)
        self.ui.setupUi()
    
    def showUI(self):
        self.mainwindow.show()


    def exec(self):
        self.app.exec_()


if __name__ == "__main__":
    ui_system = UISystem(sys.argv)
    ui_system.setUI(Ui_Main)
    ui_system.showUI()
    ui_system.exec()
    
    