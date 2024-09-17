import json
from logging import getLogger

from lib.const import CONNECT_TO_SERVER_RESPONSE, RUN_NORMALLY, SERVER_ERROR
from websockets.exceptions import ConnectionClosedError as CloseError
from websockets.exceptions import ConnectionClosedOK as CloseOK
from websockets.sync import client

logger = getLogger(__name__)


class Connection:
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.conn = None
        self.closing = False
        self.setServer("127.0.0.1", 7777)

    def setServer(self, ip: str, port: int) -> None:
        logger.info(f"setServer {ip}:{port}")
        self.ip = ip
        self.port = port

    def connect(self) -> None:
        logger.info("try to connect to server")
        if self.conn is not None:
            self.close()
        try:
            self.conn = client.connect(f"ws://{self.ip}:{self.port}")
            logger.info("connect to server successfully")
            self.ourchat.triggerEvent(
                {"code": CONNECT_TO_SERVER_RESPONSE, "status": RUN_NORMALLY}
            )
        except Exception as e:
            self.conn = None
            logger.warning(f"connect to server failed: {str(e)}")
            self.ourchat.triggerEvent(
                {
                    "code": CONNECT_TO_SERVER_RESPONSE,
                    "status": SERVER_ERROR,
                    "error_msg": str(e),
                }
            )

    def send(self, data: dict) -> None:
        json_str = json.dumps(data)
        self.conn.send(json_str)

    def recv(self) -> None:
        logger.info("begin to receive message")
        while True:
            try:
                message = self.conn.recv()
                data = json.loads(message)
                logger.info("receive message")
                logger.debug(f"receive message: {data}")
                self.ourchat.triggerEvent(data)
            except CloseError as ce:
                logger.warning(f"connection close error: {str(ce)}")
                if self.closing:
                    self.closing = False
                    return
                self.conn = None
                flag = False
                times = 1
                while (
                    not flag
                    and times <= self.ourchat.config["server"]["reconnection_attempt"]
                ):
                    flag = self.connect()[0]
                    logger.info(f"reconnect... ({times})")
                    times += 1
                if flag:
                    logger.info("reconnect successfully")
                    continue
                logger.info("server disconnect, restart")
                self.ourchat.runInMainThread(
                    lambda: self.ourchat.restart(self.ourchat.language["disconnect"])
                )
                return
            except CloseOK:
                if not self.closing:
                    logger.info("connection has been close with CLOSEOK by server")
                    self.ourchat.runInMainThread(
                        lambda: self.ourchat.restart(
                            self.ourchat.language["server_shutdown"]
                        )
                    )
                else:
                    logger.info("connection has been close with CLOSEOK")
                    self.closing = False
                return
            except Exception as e:
                logger.warning(f"unknown error: {str(e)}")
                return

    def close(self) -> None:
        logger.info("close connection")
        self.closing = True
        if self.conn is None:
            return
        self.conn.close()
        self.conn = None

    def getConnectionStatus(self) -> bool:
        return self.conn is not None
