import os
from typing import Final

RABBITMQ_HOST: Final[str] = os.environ.get("RABBITMQ_HOST", "rabbitmq")
RABBITMQ_PORT: Final[int] = int(os.environ.get("RABBITMQ_PORT", 5672))
RABBITMQ_USER: Final[str] = os.environ.get("RABBITMQ_USER", "guest")
RABBITMQ_PASS: Final[str] = os.environ.get("RABBITMQ_PASS", "guest")
RABBITMQ_VHOST: Final[str] = os.environ.get("RABBITMQ_VHOST", "/")
QUEUE_NAME: Final[str] = os.environ.get("AMQP_QUEUE", "sensor_queue")

WINDOW_SIZE: Final[int] = 10 
RECONNECT_DELAY_S: Final[int] = 10
CLI_CHECK_INTERVAL_S: Final[float] = 1.0

LOG_FORMAT: Final[str] = "%(asctime)s - %(levelname)s - [%(name)s] - %(message)s"
LOG_DATE_FORMAT: Final[str] = "%Y-%m-%d %H:%M:%S"