import ui,socket,time
from PyQt5.QtWidgets import QMainWindow, QApplication
from threading import Thread

class Ui(ui.Ui_MainWindow):
	def __init__(self):
		super().__init__()
		self.mainwindow = None
		self.mainsystem = None

	def setupUi(self):
		super().setupUi(self.mainwindow)

	def showUi(self):
		self.app = QApplication([])
		self.mainwindow = QMainWindow()
		self.setupUi()
		self.bind()
		self.mainwindow.show()
		self.app.exec()

	def bind(self):
		self.send.clicked.connect(self.sendMessage)
		self.remove.clicked.connect(self.removeIp)

	def sendMessage(self):
		if self.ip_list.currentItem() is None:
			return
		ip = self.ip_list.currentItem().text()
		msg = self.edit.toPlainText()
		self.mainsystem.send(ip,msg)

	def UpdateIp(self):
		self.ip_list.clear()
		ips = [ip for ip in self.mainsystem.ips.keys()]
		self.ip_list.addItems(ips)

	def showMsg(self,msg):
		t = time.gmtime()
		self.message.append(f"[{time.strftime('%H/%M/%S')}]{msg}\n")

	def removeIp(self):
		if self.ip_list.currentItem() is None:
			return
		ip = self.ip_list.currentItem().text()
		self.mainsystem.closeConnection(ip)


class Main:
	def __init__(self):
		self.socket = None
		self.ips = {}
		self.ui = None

	def bind(self):
		self.socket = socket.socket()
		self.socket.bind(("127.0.0.1",11451))
		self.socket.listen(10)
	
	def listen(self):
		while True:
			print("waiting...")
			c,addr = self.socket.accept()
			print(c,addr)
			self.ips[f"{addr[0]}:{addr[1]}"] = c
			self.ui.UpdateIp()
			self.ui.showMsg(f"[{addr[0]}:{addr[1]}] 已连接")
			Thread(target=self.recv,args=(f"{addr[0]}:{addr[1]}",),daemon=True).start()

	def send(self,ip,message):
		self.ips[ip].send(message.encode("utf-8"))
		self.ui.showMsg(f' 将{message}发送至[{ip}]')

	def recv(self,ip):
		while ip in self.ips:
			c = self.ips[ip]
			msg = c.recv(1024).decode("utf-8")
			if msg == "":
				continue
			self.ui.showMsg(f"[{ip}] {msg}")

	def closeConnection(self,ip):
		try:
			self.ips[ip].close()
			self.ips.pop(ip)
			self.ui.UpdateIp()
		except Exception as e:
			self.ui.showMsg(e)

main = Main()
main.bind()
ui = Ui()
ui.mainsystem = main
main.ui = ui

listen = Thread(target=main.listen,daemon=True)
listen.start()
ui.showUi()