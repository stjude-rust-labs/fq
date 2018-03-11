import re
from enum import Enum


class ValidationLevel:
    MINIMUM = 1
    LOW = 2
    HIGH = 3


class BaseSingleReadValidator:
    """Base validator for a single read, should not be called directly. This class
    is meant to be used as an abstract class for all single read FastQ validations.
    
    Args:
        level(ValidationLevel): level at which to call the validator.
        name(str): name of the error.
    """

    def __init__(self, level: ValidationLevel, name: str):
        self.level = level
        self.name = name

    def validate(self, read):
        raise NotImplementedError(
            f"'validate' not implemented for {self.__class__.__name__}")


class PluslineValidator(BaseSingleReadValidator):
    """Validates that the plusline of the FastQ file is correctly set to '+'."""

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args, level=ValidationLevel.MINIMUM, name="S001", **kwargs)

    def validate(self, read):
        if not read.plusline or read.plusline != '+':
            return False, f"The plusline is not formatted correctly. It's possible this is a " \
                          f"FastA file or that the reads are not correctly formed."

        return True, None


class AlphabetValidator(BaseSingleReadValidator):
    """Verifies that all reads are in the AGCTN dictionary."""

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args, level=ValidationLevel.LOW, name="S002", **kwargs)

    def validate(self, read):
        if re.search("[^ACGTNacgtn]", read.sequence):
            return False, f'Non-ACTGN base found in sequence {read.sequence}'

        return True, None


class ReadnameValidator(BaseSingleReadValidator):
    """Validates that a readname is well-formed (locally, not globally) for
    errors like duplicate read names."""

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args, level=ValidationLevel.HIGH, name="S003", **kwargs)

    def validate(self, read):
        if not read.name.startswith("@"):
            return False, 'Read name must start with @'

        return True, None


class BasePairedReadValidator:
    """Base validator for paired reads, should not be called directly. This class
    is meant to be used as an abstract class for all paired read FastQ validations.
    
    Args:
        level(ValidationLevel): level at which to call the validator.
        name(str): name of the error.
    """

    def __init__(self, level: ValidationLevel, name: str):
        self.level = level
        self.name = name

    def validate(self, readone, readtwo):
        raise NotImplementedError(
            f"'validate' not implemented for {self.__class__.__name__}")


class PairedReadnameValidator(BasePairedReadValidator):
    """Validates that a pair of readnames are well-formed."""

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args, level=ValidationLevel.LOW, name="P001", **kwargs)

    def validate(self, readone, readtwo):
        if not readone.name == readtwo.name:
            return False, 'Read names do not match!'

        return True, None