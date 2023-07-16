import socket,json

class Client():
	def __init__(self):
		self.server = None
		self.ocid = None

	def tryConnectToServer(self,ip,port):
		try:
			s = socket.socket()
			s.connect((ip,port))
			self.server = s
		except Exception as e:
			pass
			# TODO 无法连接至服务器

	def sendMessage(self,msg_data):
		json_str = json.dumps(msg_data)
		self.server.send(json_str.encode("utf-8"))

	def recvMessage(self):
		while True:
			json_str = self.server.recv(1024).decode("utf-8")
			if json_str == "":
				continue
			self.analysisMessage(json.loads(json_str))

	def analysisMessage(self,msg_data):
		print(msg_data)

if __name__ == "__main__":
	c = Client()
	c.tryConnectToServer("127.0.0.1",11451)
