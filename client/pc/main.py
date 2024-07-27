from uiSystem import UISystem
from ui_logic.main import Ui_Main
import sys

ui_system = UISystem(sys.argv)
ui_system.setUI(Ui_Main)
ui_system.showUI()
ui_system.exec()