import re
from enum import Enum

class ValidationLevel:
    NONE = 1
    LOW = 2
    HIGH = 3

class ReadValidator:

    def __init__(self, level: ValidationLevel):
        self.level = level

    def validate(self, read):
        still_valid, err = True, None

        if still_valid and self.level >= ValidationLevel.LOW:
            if not read.name.startswith("@"):
                still_valid, err = False, 'Read name must start with @'

        # All sequence bases must be ACGTN
        if still_valid and self.level >= ValidationLevel.LOW:
            if re.search("[^ACGTNacgtn]", read.sequence):
                still_valid, err = False, f'Non-ACTGN base found in sequence {read.sequence}'


        if still_valid and self.level >= ValidationLevel.HIGH:
            pass

        return still_valid, err