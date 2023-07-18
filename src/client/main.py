import socket, json, hashlib, asyncio,sys
from uiSystem import MainUi, UiSystem, Login
from time import time
from PyQt5.QtWidgets import QMessageBox,QApplication


class Client:
	def __init__(self):
		self.server = None
		self.ocid = None
		self.close = False
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

	async def recvMessage(self):
		while not getattr(self.server, "_closed") and not self.close:
			json_str = await self.server.recv(1024).decode("utf-8")
			if json_str == "":
				continue
			print(json_str)
			self.analysisMessage(json.loads(json_str))
			await asyncio.wait(1)

	async def listenSocket(self):
		while not self.close:
			while True:
				connect = await self.tryConnectToServer("127.0.0.1", 11451)
				if connect:
					break
			print("连接成功")
			await self.recvMessage()
	def analysisMessage(self, data: dict):
		msg_type = msg["code"]
		msg_time = msg["time"]
		msg_data = msg["data"]
		if msg_type == 7:  # 登录返回信息	
			if msg_data["state"] == 0:
				self.ocid = msg_data["ocid"]
				self.uisystem.showUi(MainUi())
				self.uisystem.closeDialog()
			elif msg_data["state"] == 1:
				QMessageBox.Critical(self.uisystem, "error", "wrong account/password.")
			elif msg_data["state"] == 2:
				QMessageBox.Critical(self.uisystem, "error", "server error")

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

	def closeClient(self):
		self.close = True

	async def showUi(self):
		app = QApplication(sys.argv)
		self.uisystem = UiSystem()
		self.uisystem.showDialog(Login())
		await app.exec()


async def run():
	client = Client()
	socket_task = asyncio.create_task(client.listenSocket())
	ui_task = asyncio.create_task(client.showUi())

	await socket_task
	await ui_task


asyncio.run(run())
