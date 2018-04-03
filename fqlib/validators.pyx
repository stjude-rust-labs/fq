# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

import re

cdef class ValidationLevel:

    def __init__(self, str level):
        # This method heavily uses Python internals, and that is okay: the convenience
        # of using Python str methods balanced with the low frequency of 
        # creating these objects means it's not something that needs to be optimized.
        if level.lower() == "minimum":
            self.level = _ValidationLevel.MINIMUM
        elif level.lower() == "low":
            self.level = _ValidationLevel.LOW
        elif level.lower() == "high":
            self.level = _ValidationLevel.HIGH
        else:
            raise ValueError(f"Unknown single read validation level: {level}.")

cdef class BaseSingleReadValidator:
    """Base validator for a single read, should not be called directly. This class
    is meant to be used as an abstract class for all single read FastQ validations.
    """

    def __init__(self, code, vlevel):
        self.error = <char*> ""
        self.code = <char*> code
        self.validation_level = ValidationLevel(vlevel)

    cpdef public cbool validate(self, FastQRead &read):
        """Abstract validation method for single read validators."""
        self.error = f"'validate' not implemented for {self.__class__.__name__}"
        return False

cdef class BasePairedReadValidator:
    """Base validator for paired reads, should not be called directly. This class
    is meant to be used as an abstract class for all paired read FastQ validations.
    """

    def __init__(self, code, vlevel):
        self.error = <char*> ""
        self.code = <char*> code
        self.validation_level = ValidationLevel(vlevel)

    cpdef public cbool validate(self, FastQRead &readone, FastQRead &readtwo):
        """Abstract validation method for paired read validators."""
        self.error = f"'validate' not implemented for {self.__class__.__name__}"
        return False


cdef class PluslineValidator(BaseSingleReadValidator):
    """Validates that the plusline of the FastQ file is correctly set to '+'."""

    def __init__(self):
        super().__init__("S001", "minimum")

    cpdef public cbool validate(self, FastQRead &read):
        if read.plusline != <char *> '+':
            self.error = <char*> "The plusline is not formatted correctly. It's possible this is a FastA file or that the reads are not correctly formed."
            return False

        return True


cdef class AlphabetValidator(BaseSingleReadValidator):
    """Verifies that all reads are composed completely of valid characters. This method
    is optimized by compiling the alphabet into a set of valid ASCII codes and
    doing a bytes-wise comparison of the string at run-time."""

    cdef public set alphabet_set

    def __init__(self, alphabet="ACGTNacgtn"):
        super().__init__("S002", "low")
        self.alphabet_set = set()
        for c in alphabet:
            self.alphabet_set.add(ord(c))

    cpdef public cbool validate(self, FastQRead &read):
        cdef str read_sequence = <str> read.sequence
        for c in read_sequence:
            if not ord(c) in self.alphabet_set:
                self.error = f'Non-valid character found in sequence {read.sequence}'
                return False
        return True


cdef class ReadnameValidator(BaseSingleReadValidator):
    """Validates that a readname is well-formed (locally, not globally) for
    errors like duplicate read names."""

    def __init__(self):
        super().__init__("S003", "high")

    cpdef public cbool validate(self, FastQRead &read):
        cdef str read_name = <str> read.name
        if not read_name.startswith("@"):
            self.error = <char*> 'Read name must start with @'
            return False

        return True


cdef class CompleteReadValidator(BaseSingleReadValidator):
    """Validates that the plusline of the FastQ file is correctly set to '+'."""

    def __init__(self):
        super().__init__("S004", "minimum")

    cpdef public cbool validate(self, FastQRead &read):
        if read.name.empty() or read.sequence.empty() or read.plusline.empty() \
           or read.quality.empty():
            self.error = <char*> "Read is not complete." 
            return False

        return True

cdef class PairedReadnameValidator(BasePairedReadValidator):
    """Validates that a pair of readnames are well-formed."""

    def __init__(self):
        super().__init__("P001", "low")

    cpdef public cbool validate(self, FastQRead &readone, FastQRead &readtwo):
        if not readone.name == readtwo.name:
            self.error = <char*> 'Read names do not match.'
            return False

        return True