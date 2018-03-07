import os
from collections import namedtuple

from .error import FastQValidationError
from .validator import ValidationLevel, ReadValidator

class FastQRead:
    __slots__ = ['name', 'sequence', 'quality']

    def __init__(self, name, sequence, quality=None):
        self.name = name
        self.sequence = sequence
        self.quality = quality

    def __repr__(self):
        return "FastQRead(name={name}, sequence={sequence}, quality={quality})".format(**self.__dict__)


class FastQFile:
    """Class used to iterate over records in a FastQ file.

    Args:
        filename(str): Location of the FastQ file.
    """

    def __init__(self, filename: str,
                 validation_level: ValidationLevel=ValidationLevel.LOW):
        self._filename = os.path.abspath(filename)
        self._handle = open(self._filename, 'r') # raises FileNotFoundError
        if isinstance(validation_level, ValidationLevel):
            self._validation_level = validation_level
        elif isinstance(validation_level, str):
            if validation_level.lower() == "high":
                self._validation_level = ValidationLevel.HIGH
            elif validation_level.lower() == 'low':
                self._validation_level == ValidationLevel.LOW
            elif validation_level.lower() == 'none' or not validation_level:
                self._validation_level = ValidationLevel.NONE
            else:
                raise RuntimeError(f"Unknown validation level: {validation_level}")
        else:
            raise RuntimeError(f"Unknown validation level: {validation_level}")

        self._validator = ReadValidator(self._validation_level)


    def __iter__(self):
        return self
 

    def __next__(self):
        return self._next_read()


    def _next_line(self):
        return self._handle.readline().strip()


    def _next_read(self):
        line_offset = 0
        read_name = self._next_line()
        if not read_name:
            raise StopIteration

        while not read_name.startswith('>') and not read_name.startswith("@"):
            read_name = self._next_line()
            if not read_name:
                raise StopIteration
            line_offset += 1

        # Read the sequence line
        read_sequence = self._next_line()
        read_quality = None

        # Peek to see if this is Fasta or Fastq.
        currpos = self._handle.tell()

        if self._next_line() == '+':
            read_quality = self._next_line()
        else:
            # Fasta, rollback file pointer
            self._handle.seek(currpos, 0)

        read = FastQRead(name=read_name, sequence=read_sequence, quality=read_quality)
        result, err = self._validator.validate(read)

        if not result:
            raise FastQValidationError(description=err, readname=read.name, filename=self._filename)

        return read


    def close(self):
        self._handle.close()


if __name__ == "__main__":
    f = FastQFile("./example/R1_mixed.fastq", validation_level=ValidationLevel.HIGH)

    i = 0
    for line in f:
        print(line)
        i += 1
        if i % 1000 == 0 and i != 0:
            break
