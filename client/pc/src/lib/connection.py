import json
from logging import getLogger
from typing import Tuple

from websockets.exceptions import ConnectionClosedError as CloseError
from websockets.exceptions import ConnectionClosedOK as CloseOK
from websockets.sync import client

logger = getLogger(__name__)


class Connection:
    def __init__(self, ourchat) -> None:
        self.ourchat = ourchat
        self.conn = None
        self.setServer("127.0.0.1", 7777)

    def setServer(self, ip: str, port: int) -> None:
        logger.info(f"setServer {ip}:{port}")
        self.ip = ip
        self.port = port

    def connect(self) -> Tuple[bool, str]:
        logger.info("try to connect to server")
        if self.conn is not None:
            self.conn.close()
            self.conn = None
        try:
            self.conn = client.connect(f"ws://{self.ip}:{self.port}")
            logger.info("connect to server successfully")
            return True, None
        except Exception as e:
            self.conn = None
            logger.warning(f"connect to server failed: {str(e)}")
            return False, str(e)

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
                self.ourchat.getMessage(data)
            except CloseError as ce:
                logger.warning(f"connection close error: {str(ce)}")
                self.conn = None
                flag = False
                times = 1
                while not flag or times <= 5:
                    flag = self.connect()
                    times += 1
                    logger.info(f"reconnect... ({times})")
                if not flag:
                    logger.info("reconnect successfully")
                    continue
                logger.info("server disconnect, restart")
                self.ourchat.runInMainThread(
                    lambda: self.ourchat.restart(self.ourchat.language["disconnect"])
                )
                return
            except CloseOK:
                logger.info("connection has been close normally")
                self.ourchat.runInMainThread(
                    lambda: self.ourchat.restart(
                        self.ourchat.language["server_shutdown"]
                    )
                )
                return
            except Exception as e:
                logger.warning(f"unknown error: {str(e)}")
                return

    def close(self) -> None:
        logger.info("close connection")
        if self.conn is None:
            return
        self.conn.close()
        self.conn = None
