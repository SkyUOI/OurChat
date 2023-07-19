import socket, json, hashlib, asyncio,sys,time
from uiSystem import MainUi, UiSystem, Login
from threading import Thread
from PyQt5.QtWidgets import QMessageBox,QApplication


class Client:
	def __init__(self):
		self.server = None
		self.ocid = None
		self.uisystem = None

	def tryConnectToServer(self, ip: str, port: int):
		try:
			s = socket.socket()
			s.connect((ip, port))
			self.server = s
			return True
		except Exception as e:
			self.server = None
			return False

	def sendMessage(self, msg_data: dict):
		json_str = json.dumps(msg_data)
		self.server.send(json_str.encode("utf-8"))

	def recvMessage(self):
		while hasattr(self.server, "_closed"):
			json_str = self.server.recv(1024).decode("utf-8")
			if json_str == "":
				continue
			self.analysisMessage(json.loads(json_str))

	def listenSocket(self):
		while True:
			connect = self.tryConnectToServer("127.0.0.1", 11451)
			if connect:
				break
		print("连接成功")
		self.recvMessage()

	def analysisMessage(self, data: dict):
		print(data)
		msg_type = data["code"]
		msg_time = data["time"]
		msg_data = data["data"]
		if msg_type == 7:  # 登录返回信息	
			if msg_data["state"] == 0:
				self.ocid = msg_data["ocid"]
				self.uisystem.task_queue.append(lambda:self.uisystem.showUi(MainUi()))
			elif msg_data["state"] == 1:
				self.uisystem.task_queue.append(lambda:QMessageBox.critical(self.uisystem, "error", "wrong account/password."))
			elif msg_data["state"] == 2:
				self.uisystem.task_queue.append(lambda:QMessageBox.critical(self.uisystem, "error", "server error"))

	def login(self, account, password):
		encrypted_password = hashlib.sha256()
		encrypted_password.update(password.encode("utf-8"))
		encrypted_password = encrypted_password.hexdigest()
		data = {
		    "code": 6,
		    "time": time.time(),
		    "data": {"account": account, "password": encrypted_password},
		}
		self.sendMessage(data)

	def showUi(self):
		app = QApplication(sys.argv)
		self.uisystem = UiSystem(self)
		self.uisystem.showUi(Login())
		app.exec()


# async def run():
# 	client = Client()
# 	socket_task = asyncio.create_task(client.listenSocket())
# 	ui_task = asyncio.create_task(client.showUi())

# 	await socket_task
# 	await ui_task


# asyncio.run(run())

# 协程 10768 K
# 线程 16352 K

client = Client()
socket_task = Thread(target=client.listenSocket,daemon=True)
socket_task.start()
client.showUi()
if client.server is not None:
	client.server.close()