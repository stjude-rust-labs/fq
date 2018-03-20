"""All validation logic for fqlib."""

#cython: infertypes=True

import re

class ValidationLevel:
    """Utility class containing the various validation levels."""

    MINIMUM = 1
    LOW = 2
    HIGH = 3

    @staticmethod
    def resolve(value):
        """Resolve an input to a ValidationLevel or throw an error.

        Args:
            value(object): Any value that may be interpretted as a ValidationLevel.

        Throws:
            ValueError: if the input cannot be parsed, a ValueError is thrown.

        Return:
            A validation level."""

        if isinstance(value, ValidationLevel):
            return value
        elif isinstance(value, str):
            if value.lower() == "high":
                return ValidationLevel.HIGH
            elif value.lower() == 'low':
                return ValidationLevel.LOW
            elif value.lower() == 'minimum' or not value:
                return ValidationLevel.MINIMUM

        raise ValueError(f"Unknown single read validation level: {value}.")


class BaseSingleReadValidator:
    """Base validator for a single read, should not be called directly. This class
    is meant to be used as an abstract class for all single read FastQ validations.
    """

    def validate(self, read):
        """Abstract validation method for single read validators."""
        raise NotImplementedError(
            f"'validate' not implemented for {self.__class__.__name__}"
        )


class PluslineValidator(BaseSingleReadValidator):
    """Validates that the plusline of the FastQ file is correctly set to '+'."""

    level = ValidationLevel.MINIMUM
    code = "S001"

    def validate(self, read):
        if not read.plusline or read.plusline != b'+':
            return False, f"The plusline is not formatted correctly. " \
                          f"It's possible this is a FastA file or that the reads "\
                          f"are not correctly formed."

        return True, None


class AlphabetValidator(BaseSingleReadValidator):
    """Verifies that all reads are composed completely of valid characters. This method 
    is optimized by compiling the alphabet into a set of valid ASCII codes and 
    doing a bytes-wise comparison of the string at run-time."""

    level = ValidationLevel.LOW
    code = "S002"

    def __init__(self, alphabet="ACGTNacgtn"):
        self.alphabet_set = set()
        for char in alphabet:
            self.alphabet_set.add(ord(char))

    def validate(self, read):
        for char in read.sequence:
            if not char in self.alphabet_set:
                return False, f'Non-valid character found in sequence {read.sequence}'

        return True, None


class ReadnameValidator(BaseSingleReadValidator):
    """Validates that a readname is well-formed (locally, not globally) for
    errors like duplicate read names."""

    level = ValidationLevel.HIGH
    code = "S003"

    def validate(self, read):
        if not read.name.startswith(b"@"):
            return False, 'Read name must start with @'

        return True, None


class CompleteReadValidator(BaseSingleReadValidator):
    """Validates that the plusline of the FastQ file is correctly set to '+'."""

    level = ValidationLevel.MINIMUM
    code = "S004"

    def validate(self, read):
        if not read.name or not read.sequence or not read.plusline or not read.quality:
            return False, f"Read is not complete."

        return True, None


class BasePairedReadValidator:
    """Base validator for paired reads, should not be called directly. This class
    is meant to be used as an abstract class for all paired read FastQ validations.
    """

    def validate(self, readone, readtwo):
        """Abstract validation method for paired read validators."""
        raise NotImplementedError(
            f"'validate' not implemented for {self.__class__.__name__}"
        )


class PairedReadnameValidator(BasePairedReadValidator):
    """Validates that a pair of readnames are well-formed."""

    level = ValidationLevel.LOW
    code = "P001"

    def validate(self, readone, readtwo):
        if not readone.name == readtwo.name:
            return False, 'Read names do not match.'

        return True, None
