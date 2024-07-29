from lib.uiSystem import UISystem
from ui_logic.main import Ui_Main
from ui_logic.login import Ui_Login
import sys

ui_system = UISystem(sys.argv)
mainwindow = ui_system.setUI(Ui_Main)
ui_system.mainwindow.setEnabled(False)
dialog = ui_system.setDialog(Ui_Login, True)
dialog.show()
ui_system.exec()
