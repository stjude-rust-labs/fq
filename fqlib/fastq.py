"""All FastQ functionality for fqlib.

This module includes all FastQ functionality. Included in the module are things
such as utilities for reading/stepping-through FastQ files and reading/stepping-through
a pair of FastQ files.
"""

__author__ = "Clay McLeod"

import os

from . import validators
from .error import SingleReadValidationError, PairedReadValidationError
from .validators import (
    ValidationLevel, BaseSingleReadValidator, BasePairedReadValidator
)

POSSIBLE_INTERLEAVES = ["/1", "/2"]


class FastQRead:
    """Lightweight class for a FastQ read.

    Attributes:
        name (str): proper name of the read in the FastQ file.
        sequence (str): sequence referred to in the read.
        plusline (str): content of the 'plusline' in the read.
        quality (str): corresponding quality of sequence in the read.
        interleave (str or None): if applicable, interleave of the read.

    Args:
        name (str): proper name of the read in the FastQ file.
        sequence (str): sequence referred to in the read.
        plusline (str): content of the 'plusline' in the read.
        quality (str): corresponding quality of sequence in the read.
    """

    __slots__ = ['name', 'sequence', 'plusline', 'quality', 'interleave']

    def __init__(self, name: str, sequence: str, plusline: str, quality: str):
        self.name = name
        self.sequence = sequence
        self.plusline = plusline
        self.quality = quality
        self.interleave = None

        # Search read name for interleave
        for interleave in POSSIBLE_INTERLEAVES:
            if self.name.endswith(interleave):
                self.name = self.name[:-len(interleave)]
                self.interleave = interleave

    def __repr__(self):
        return f"FastQRead(name={self.name}, sequence={self.sequence}, " \
               f"plusline={self.plusline}, quality={self.quality}, " \
               f"interleave={self.interleave})"


class _FileReader:
    """Abstraction class for a single file to be open and read. This is simply
    a convenience class.

    Attributes:
        name (str): Filename to be used to open the file.
        basename (str): Basename of the file name.
        handle (File): Object of file opened using file name, readable only.
        lineno (int): Current line of the file.

    Args:
        name(str): Filename to be used to open the file.
    """

    __slots__ = ['name', 'basename', 'handle', 'lineno']

    def __init__(self, name: str):
        self.name = os.path.abspath(name)
        self.basename = os.path.basename(self.name)
        self.handle = open(self.name, 'r')
        self.lineno = 0

    def getlines(self, num_lines: int = 1):
        """Get :obj:`num_lines` number of lines from a file.

        Todo:
            - Probably a cleaner way to do this.

        Returns:
            list: `num_lines` lines of a file, stripped of whitespace."""

        lines_seen = 0
        results = []

        while lines_seen < num_lines:
            try:
                result = next(self.handle).strip()
            except StopIteration:
                break

            lines_seen += 1
            results.append(result)
            self.lineno += 1

        while lines_seen < num_lines:
            results.append(None)
            lines_seen += 1

        return results


class FastQFile:
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

    def __init__(
        self,
        filename: str,
        validation_level: ValidationLevel = ValidationLevel.HIGH,
        lint_mode: str = "error"
    ):

        self.file = _FileReader(filename)
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

    def next_read(self):
        """Naively read the FastQ read. If something goes awry, expect it to get caught
        in the validators.

        Returns:
            FastQRead: a read from the FastQ file

        Raises:
            Error: multiple errors may be thrown, especially FastQ validation errors.
        """

        [rname, rsequence, rplusline, rquality] = self.file.getlines(4)  # pylint: disable=E0632

        # only check against read name because if any of the others are none, that
        # should signal an incomplete read, not running out of reads in the file.
        if not rname:
            raise StopIteration

        read = FastQRead(
            name=rname,
            sequence=rsequence,
            plusline=rplusline,
            quality=rquality
        )

        for validator in self.validators:
            result, description = validator.validate(read)
            if not result or description:
                if self.lint_mode == "error":
                    raise SingleReadValidationError(
                        description=description,
                        readname=read.name,
                        filename=self.file.basename,
                        lineno=self.file.lineno
                    )
                elif self.lint_mode == "report":
                    print(
                        f"{self.file.basename}:{validator.code}:{self.file.lineno}: " \
                        f"{description}"
                    )
                else:
                    raise NotImplementedError(
                        f"Not implemented for lint mode: {self.lint_mode}"
                    )

        return read

    def close(self):
        """Closes the file handle."""
        self.file.handle.close()


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
                        f"{self.read_one_fastqfile.file.basename}:{validator.code}:" \
                        f"{self._readno}: {description}"
                    )
                    print(
                        f"{self.read_two_fastqfile.file.basename}:{validator.code}:" \
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
