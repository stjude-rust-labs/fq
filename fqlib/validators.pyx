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

#
#
#class AlphabetValidator(BaseSingleReadValidator):
#    """Verifies that all reads are composed completely of valid characters. This method
#    is optimized by compiling the alphabet into a set of valid ASCII codes and
#    doing a bytes-wise comparison of the string at run-time."""
#
#    level = ValidationLevel.LOW
#    code = "S002"
#
#    def __init__(self, alphabet="ACGTNacgtn"):
#        self.alphabet_set = set()
#        for char in alphabet:
#            self.alphabet_set.add(ord(char))
#
#    def validate(self, read):
#        for char in read.sequence:
#            if not char in self.alphabet_set:
#                return False, f'Non-valid character found in sequence {read.sequence}'
#
#        return True, None
#
#
#class ReadnameValidator(BaseSingleReadValidator):
#    """Validates that a readname is well-formed (locally, not globally) for
#    errors like duplicate read names."""
#
#    level = ValidationLevel.HIGH
#    code = "S003"
#
#    def validate(self, read):
#        if not read.name.startswith(b"@"):
#            return False, 'Read name must start with @'
#
#        return True, None
#
#
#class CompleteReadValidator(BaseSingleReadValidator):
#    """Validates that the plusline of the FastQ file is correctly set to '+'."""
#
#    level = ValidationLevel.MINIMUM
#    code = "S004"
#
#    def validate(self, read):
#        if not read.name or not read.sequence or not read.plusline or not read.quality:
#            return False, f"Read is not complete."
#
#        return True, None



cdef class PairedReadnameValidator(BasePairedReadValidator):
    """Validates that a pair of readnames are well-formed."""

    def __init__(self):
        super().__init__("P001", "low")

    cpdef public cbool validate(self, FastQRead &readone, FastQRead &readtwo):
        if not readone.name == readtwo.name:
            self.error = <char*> 'Read names do not match.'
            return False

        return True