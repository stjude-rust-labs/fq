"Timer/Stopwatch utility for seeing how quickly reads are being processed."

import time
import logging


class Timer:
    """A utility timer/stopwatch in the style of a context manager used
    to monitor how quickly reads are being processed.

    Usage:
        with Timer() as t:
            ... process reads ...
            if num_reads % frequency_to_print == 0:
                t.update(num_reads)
    """

    def __init__(self):
        self.start_time = None
        self.total_reads = -1
        self.laps = []

    def __enter__(self):
        """Required for context manager."""
        self.start_time = time.clock()
        self.total_reads = 0
        self.laps = [(self.start_time, 0)]
        return self

    def __exit__(self, atype, value, traceback):
        """Required for context manager."""
        return False

    def _lap(self):
        """Method to lap in the style of a stopwatch"""
        current_lap = (time.clock(), self.total_reads)
        last_lap = self.laps[-1]
        self.laps.append(current_lap)
        return (current_lap[0] - last_lap[0], current_lap[1] - last_lap[1])

    def update(self, read_count):
        """Update reads processed, create a new lap on the stopwatch, output result."""
        self.total_reads += read_count
        (t, r) = self._lap()  # pylint: disable=C0103
        logging.info(
            f"Processed {r} reads in {t:.2f} seconds ({r/t:.0f} reads/sec). " \
            f"Total: {self.total_reads} reads."
        )
