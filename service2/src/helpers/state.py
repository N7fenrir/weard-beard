import asyncio
from collections import deque
from typing import Dict, Any, Optional
from . import config

SensorData = Dict[str, Any]
StatsDict = Dict[str, SensorData]

sensor_stats: StatsDict = {}

stats_lock: asyncio.Lock = asyncio.Lock()

log_filter_type: Optional[str] = None

def initialize_sensor_type(sensor_type: str) -> None:
    """Initializes the state dictionary for a new sensor type."""
    if sensor_type not in sensor_stats:
        sensor_stats[sensor_type] = {
            "values": deque(maxlen=config.WINDOW_SIZE),
            "mean": None,
            "stdev": None,
            "count": 0
        }

def get_log_filter() -> Optional[str]:
    """Gets the current log filter."""
    return log_filter_type

def set_log_filter(sensor_type: Optional[str]) -> None:
    """Sets the log filter. None for all."""
    global log_filter_type
    log_filter_type = sensor_type