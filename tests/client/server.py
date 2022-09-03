import socket,tkinter,json,time
from tkinter import ttk
from threading import Thread

data = None
class Server(Thread):
    def __init__(self):
        self.s = socket.socket()
        self.s.bind(("127.0.0.1",54088))
        self.ips = {}
        self.s.listen(5)
        super().__init__()
    
    def run(self):
        global data
        while True:
            a,b = self.s.accept()
            self.ips[f"{b[0]}:{b[1]}"] = a
            print(b)
            Thread(target=self.read,args=(f"{b[0]}:{b[1]}",),daemon=True).start()
    
    def send(self,ip,text):
        self.ips[ip].send(text.encode("utf-8"))
    
    def read(self,ip):
        t = self.ips[ip]
        try:
            while True:
                msg = t.recv(1024).decode("utf-8")
                if msg == "":
                    continue
                print(ip,msg)
        except:
            self.ips.pop(ip)

def window(obj):
    global data
    root = tkinter.Tk()
    ip = tkinter.Entry(root,width=60)
    data = ttk.Combobox(root,width=60)
    data.config(
                    values=[
                        '{"code" : 0, "time" : ' + str(int(time.time())) + ' , "data" : {"cid" : 1,"sender_id" : "ocid","msg" : "文本信息"}}',
                        '{"code" : 5,"data" : {"state" : 0,"ocId" : "oc_123456"}}',
                        '{"code" : 5,"data" : {"state" : 1,"ocId" : ""}}',
                        '{"code" : 7,"data" : {"state" : 0}}',
                        '{"code" : 7,"data" : {"state" : 1}}'
                    ]
                )
    ip.pack()
    data.pack()
    tkinter.Button(root,text="发送",command=lambda:obj.send(ip.get(),data.get())).pack()
    root.mainloop()

t = Server()
t.setDaemon(True)
t.start()

window(t)