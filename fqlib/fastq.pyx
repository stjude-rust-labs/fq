# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

import os

from . import validators
from .error import SingleReadValidationError, PairedReadValidationError
from .validators import (
    ValidationLevel, BaseSingleReadValidator, BasePairedReadValidator
)


cdef string[2] POSSIBLE_INTERLEAVES
POSSIBLE_INTERLEAVES[:] = [<char*> "/1",<char*> "/2"]


cpdef FastQRead fqread_init(name: string, sequence: string, plusline: string, quality: string):
    cdef FastQRead result

    result.name = name
    result.sequence = sequence
    result.plusline = plusline
    result.quality = quality
    result.interleave = <char*> ""

    cdef int i = 0

    for i in range(len(POSSIBLE_INTERLEAVES)):
        interleave = POSSIBLE_INTERLEAVES[i]
        if ends_with(result.name, interleave):
            result.name = result.name.substr(0, result.name.size() - 2)
            result.interleave = result.name.substr(result.name.size() - 2, 2)

    return result


cpdef FastQRead fqread_generate():
    cdef FastQRead result

    cdef instrument = "illumina1"
    cdef run_number = "1"
    cdef flowcell = "AABBCC"
    cdef lane = "1"
    cdef tile = "1"
    cdef x_pos = "1"
    cdef y_pos = "1"
    cdef sequence = "AAAAAAAAAACCCCCCCCCGGGGGGGGGGTTTTTTTTTT"
    cdef plusline = "+"
    cdef quality =  "JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ"

    return fqread_init(
        f"{instrument}:{run_number}:{flowcell}:{lane}:{tile}:{x_pos}:{y_pos}",
        sequence,
        plusline,
        quality
    )


cpdef str fqread_repr(FastQRead read):
    return f"FastQRead(name='{read.name)}', "\
            f"sequence='{read.sequence}', " \
            f"plusline='{read.plusline}', " \
            f"quality='{read.quality}', " \
            f"interleave='{read.interleave}')"


cdef class CFileReader:
    """Utility class used internally to read files using the C API. This
    class is meant to be used primarily as a building block for the
    FastQFile python class."""

    def __init__(self, filename: str):
        self.filename = filename
        self.handle = fopen(self.filename, "r")
        self.lineno = 0
        self.rlen = 0
        self.line = NULL

    cdef string read_line(self):
        cdef string result
        nread = getline(&self.line, &self.rlen, self.handle)
        self.lineno = self.lineno + 1
        if nread == -1:
            return result
        result = string(self.line)
        result = result.substr(0, result.size() - 1) # remove newline
        return result

    cdef void close(self):
        if self.handle:
            fclose(self.handle)
            self.handle = NULL


cdef class FastQFile:
    """Class used to iterate over records in a FastQ file. Validation will be
    performed based on the level of validation set in the constructor. Eventually,
    we will add functionality to efficiently unpack only the needed parts of the
    FastQ file.

    Args:
        filename (str): Path to the FastQ file.
        validation_level (validators.ValidationLevel): Degree to which the FastQ file
            should be validated.
        lint_mode (str): Must be one of the lint modes supported

    Raises:
        FileNotFoundError, ValueError
    """

    cdef CFileReader cfile_handle
    cdef string filename
    cdef string basename
    cdef object vlevel
    cdef str lint_mode

    def __init__(
        self,
        filename: str,
        validation_level: ValidationLevel = ValidationLevel.HIGH,
        lint_mode: str = "error"
    ):

        self.filename = filename
        self.basename = os.path.basename(self.filename)
        self.cfile_handle = CFileReader(filename)
        self.vlevel = ValidationLevel.resolve(validation_level)

        self.lint_mode = lint_mode

        self.validators = [
            v() for (k, v) in validators.__dict__.items()
            if not k.startswith("Base") and isinstance(v, type) and
            issubclass(v, BaseSingleReadValidator) and v.level <= self.vlevel
        ]

    def __iter__(self):
        """Iterator methods."""
        return self

    def __next__(self):
        """Iterator methods."""
        return self.next_read()

    cdef next_read(self):
        """Naively read the FastQ read. If something goes awry, expect it to get caught
        in the validators.

        Returns:
            FastQRead: a read from the FastQ file

        Raises:
            Error: multiple errors may be thrown, especially FastQ validation errors.
        """

        cdef string rname
        cdef string rsequence
        cdef string rplusline
        cdef string rquality
        cdef FastQRead read

        rname = self.cfile_handle.read_line()
        rsequence = self.cfile_handle.read_line()
        rplusline = self.cfile_handle.read_line()
        rquality = self.cfile_handle.read_line()

        # only check against read name because if any of the others are none, that
        # should signal an incomplete read, not running out of reads in the file.
        if rname == <char*> "":
            raise StopIteration

        read = fqread_init(rname, rsequence, rplusline, rquality)

        for validator in self.validators:
            result, description = validator.validate(read)
            if not result or description:
                if self.lint_mode == "error":
                    raise SingleReadValidationError(
                        description=description,
                        readname=read.name,
                        filename=self.basename,
                        lineno=self.cfile_handle.lineno
                    )
                elif self.lint_mode == "report":
                    print(
                        f"{self.basename}:{validator.code}:{self.cfile_handle.lineno}: " \
                        f"{description}"
                    )
                else:
                    raise NotImplementedError(
                        f"Not implemented for lint mode: {self.lint_mode}"
                    )

        return read

    def close(self):
        """Closes the file handle."""
        self.cfile_handle.close()


class PairedFastQFiles:
    """A class that steps through two FastQFiles at the same time and validates them
    from a global perspective.

    Args:
        read_one_filename (str): Path to the read one file.
        read_two_filename (str): Path to the read two file.
        lint_mode (str): Linting in 'error' or 'report' mode.
        single_read_validation_level (str): Validation level for the single read errors.
        paired_read_validation_level (str): Validation level for the paired read errors.
    """

    def __init__(
        self,
        read_one_filename: str,
        read_two_filename: str,
        lint_mode: str = "error",
        single_read_validation_level: str = "low",
        paired_read_validation_level: str = "low"
    ):
        self.read_one_fastqfile = FastQFile(
            read_one_filename,
            validation_level=single_read_validation_level,
            lint_mode=lint_mode
        )
        self.read_two_fastqfile = FastQFile(
            read_two_filename,
            validation_level=single_read_validation_level,
            lint_mode=lint_mode
        )
        self.lint_mode = lint_mode
        self.vlevel = ValidationLevel.resolve(paired_read_validation_level)

        self._readno = 0
        self.validators = [
            v() for (k, v) in validators.__dict__.items()
            if not k.startswith("Base") and isinstance(v, type) and
            issubclass(v, BasePairedReadValidator) and v.level <= self.vlevel
        ]

    def __iter__(self):
        """Iterator methods."""
        return self

    def __next__(self):
        """Iterator methods."""
        result = self.next_readpair()
        self._readno += 1
        return result

    def next_readpair(self):
        """Get the next pair of reads from both files.

        Returns:
            (FastQRead, FastQRead): tuple of both FastQRead objects, one from each file.

        Raises:
            Error: multiple errors may be thrown, especially FastQ validation errors
        """

        read_one = self.read_one_fastqfile.next_read()
        read_two = self.read_two_fastqfile.next_read()

        # Must be "and". If one is None and not the other, that's a mismatched FastQ
        # read in one of the files.
        if not read_one and not read_two:
            raise StopIteration

        for validator in self.validators:
            result, description = validator.validate(read_one, read_two)
            if not result or description:
                if self.lint_mode == "error":
                    raise PairedReadValidationError(
                        description=description,
                        read_one=read_one,
                        read_two=read_two,
                        read_pairno=self._readno,
                        read_one_fastqfile=self.read_one_fastqfile,
                        read_two_fastqfile=self.read_two_fastqfile
                    )
                elif self.lint_mode == "report":
                    print(
                        f"{self.read_one_fastqfile.basename}:{validator.code}:" \
                        f"{self._readno}: {description}"
                    )
                    print(
                        f"{self.read_two_fastqfile.basename}:{validator.code}:" \
                        f"{self._readno}: {description}"
                    )
                else:
                    raise NotImplementedError(
                        f"Not implemented for lint mode: {self.lint_mode}"
                    )

        return (read_one, read_two)

    def get_validators(self):
        """Returns a tuple with (SingleReadValidators, PairedReadValidators)"""
        sr_validators = [
            (v.code, v.__class__.__name__)
            for v in self.read_one_fastqfile.validators
        ]
        pr_validators = [
            (v.code, v.__class__.__name__) for v in self.validators
        ]
        return (sr_validators, pr_validators)

    def close(self):
        """Close both FastQ files."""
        self.read_one_fastqfile.close()
        self.read_two_fastqfile.close()
