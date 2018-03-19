import time
import logging

class Timer:

    def __enter__(self):
        self.start_time = time.clock()
        self.total_reads = 0
        self.laps = [(self.start_time, 0)]
        return self
    
    def __exit__(self ,type, value, traceback):
        return True

    def _lap(self):
        curr_lap = (time.clock(), self.total_reads)
        last_lap = self.laps[-1] 
        self.laps.append(curr_lap)
        return (curr_lap[0] - last_lap[0], curr_lap[1] - last_lap[1])

    def update(self, read_count):
        self.total_reads += read_count
        (t, r) = self._lap()
        logging.info(f"Processed {r} reads in {t:.2f} seconds ({r/t:.0f} reads/sec). Total: {self.total_reads} reads.")