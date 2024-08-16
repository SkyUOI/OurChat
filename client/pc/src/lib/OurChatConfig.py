import json
import os
import re
from copy import deepcopy
from logging import CRITICAL, DEBUG, ERROR, INFO, WARNING, getLogger
from typing import Any, Union

logger = getLogger(__name__)


class OurChatConfig:
    def __init__(self, path: str = "./", filename: str = "config.json") -> None:
        self.path = path
        self.filename = filename
        self.read()

    def defaultConfig(self) -> dict:
        return {
            "server": {"ip": "127.0.0.1", "port": 7777, "reconnection_attempt": 5},
            "general": {"theme": "dark_amber", "language": "en-us"},
            "advanced": {"log_level": INFO, "log_saving_limit": 30},
        }

    def write(self) -> None:
        logger.info(f"write config to {os.path.join(self.path,self.filename)}")
        self.check()
        with open(os.path.join(self.path, self.filename), "w") as f:
            json.dump(self.config, f, indent=1)

    def read(self) -> None:
        logger.info(f"read config from {os.path.join(self.path,self.filename)}")
        if not os.path.exists(os.path.join(self.path, self.filename)):
            logger.info(
                f"{os.path.join(self.path,self.filename)} not exist, use default config"
            )
            self.config = self.defaultConfig()
            self.write()
        with open(os.path.join(self.path, self.filename), "r") as f:
            self.config = json.load(f)
        self.check()

    def __getitem__(self, key: str) -> Union[Any, None]:
        if key not in self.config.keys():
            default = self.defaultConfig()
            if key in default.keys():
                self[key] = default[key]
                return self[key]
            else:
                return None
        return self.config[key]

    def __setitem__(self, key: str, value: Any) -> None:
        self.config[key] = value

    def checkType(
        self,
        value_type: Any | None = None,
        default_value: Any | None = None,
        config: Any | None = None,
    ) -> Any:
        if value_type is None:
            value_type = {
                "server": {"ip": str, "port": int, "reconnection_attempt": int},
                "general": {"theme": str, "language": str},
                "advanced": {"log_level": int, "log_saving_limit": int},
            }
            default_value = self.defaultConfig()
            config = self.config

        if isinstance(default_value, dict):
            for key in value_type.keys():
                if key not in config.keys():
                    config[key] = default_value[key]
                else:
                    config[key] = self.checkType(
                        value_type[key], default_value[key], config[key]
                    )
            return config
        else:
            if isinstance(config, value_type):
                return config
            else:
                logger.warning("config type error")
                return default_value

    def check(self) -> None:
        logger.info("check config")
        self.checkType()

        default = self.defaultConfig()
        ip = re.match(
            "^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
            self["server"]["ip"],
        )
        if ip is None:
            self["server"]["ip"] = default["server"]["ip"]
        if self["server"]["port"] < 1 or self["server"]["port"] > 65535:
            self["server"]["port"] = default["server"]["port"]

        themes = os.listdir("theme")
        if self["general"]["theme"] not in themes:
            if default["general"]["theme"] in themes:
                self["general"]["theme"] = default["general"]["theme"]
            elif len(themes) > 0:
                self["general"]["theme"] = themes[0]
            else:
                self["general"]["theme"] = None

        languages = [language.replace(".lang", "") for language in os.listdir("lang")]
        if self["general"]["language"] not in languages:
            if default["general"]["language"] in languages:
                self["general"]["language"] = default["general"]["language"]
            elif len(languages) > 0:
                self["general"]["language"] = languages[0]
            else:
                self["general"]["language"] = None

        if self["advanced"]["log_level"] not in [DEBUG, INFO, WARNING, ERROR, CRITICAL]:
            self["advanced"]["log_level"] = default["advanced"]["log_level"]

        if (
            self["advanced"]["log_saving_limit"] < 1
            and self["advanced"]["log_saving_limit"] != -1
        ):
            self["advanced"]["log_saving_limit"] = default["advanced"][
                "log_saving_limit"
            ]

    def setConfig(self, config: Union[Any, dict]) -> None:
        if isinstance(config, dict):
            self.config = deepcopy(config)
        elif isinstance(config, OurChatConfig):
            self.config = deepcopy(config.config)

    def compareConfig(self, config: Union[Any, dict]) -> bool:
        if isinstance(config, dict):
            return self.config == config
        elif isinstance(config, OurChatConfig):
            return self.config == config.config
