import statistics
from typing import Deque, Dict, Optional

def calculate_stats(values: Deque[float]) -> Dict[str, Optional[float]]:
    count = len(values)
    if count == 0:
        return {"mean": None, "stdev": None}

    mean = statistics.mean(values)
    stdev: Optional[float] = None
    if count > 1:
        try:
            stdev = statistics.stdev(values)
        except statistics.StatisticsError:
            stdev = None
    return {"mean": mean, "stdev": stdev}