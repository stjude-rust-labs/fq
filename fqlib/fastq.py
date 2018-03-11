import os
from collections import namedtuple, defaultdict

from . import validators
from .error import SingleReadValidationError, PairedReadValidationError
ValidationLevel = validators.ValidationLevel
BaseSingleReadValidator = validators.BaseSingleReadValidator
BasePairedReadValidator = validators.BasePairedReadValidator


class FastQRead:
    __slots__ = ['name', 'sequence', 'plusline', 'quality', 'interleave']
    _possible_interleaves = ["/1", "/2"]

    def __init__(self, name, sequence, plusline, quality):
        self.name = name
        self.sequence = sequence
        self.plusline = plusline
        self.quality = quality
        self.interleave = None

        # Search read name for interleave
        for interleave in self._possible_interleaves:
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
    
    Args:
        name(str): Filename to be used to open the file.
    """

    __slots__ = ['name', 'basename', 'handle', '_lineno']

    def __init__(self, name: str):
        self.name = os.path.abspath(name)
        self.basename = os.path.basename(self.name)
        self.handle = open(self.name, 'r')
        self._lineno = 0

    def getlines(self, num_lines: int = 1):
        """Get `num_lines` number of lines from a file.
        
        Todo:
            - Probably a cleaner way to do this.
           
        Return:
            list: `num_lines` lines of a file, stripped of whitespace."""

        _i = 0
        results = []

        while _i < num_lines:
            _i += 1
            results.append(next(self.handle).strip())
            self._lineno += 1

        return results


class FastQFile:
    """Class used to iterate over records in a FastQ file. Validation will be
    performed based on the level of validation set in the constructor. Eventually,
    we will add functionality to efficiently unpack only the needed parts of the
    FastQ file.

    Args:
        filename(str): Path to the FastQ file.
        validation_level(validators.ValidationLevel): Degree to which the FastQ file
                       should be validated.
        lint_mode(str): Must be one of the lint modes supported 

    Raises:
        FileNotFoundError, ValueError
    """

    def __init__(self,
                 filename: str,
                 validation_level: ValidationLevel = ValidationLevel.HIGH,
                 lint_mode: str = "error"):

        self.file = _FileReader(filename)
        self.vlevel = ValidationLevel.resolve(validation_level)
        self.lint_mode = lint_mode

        self._validators = [
            v() for (k, v) in validators.__dict__.items()
            if not k.startswith("Base") and isinstance(v, type) and issubclass(
                v, BaseSingleReadValidator) and v.level <= self.vlevel
        ]

    def __iter__(self):
        """Iterator methods."""
        return self

    def __next__(self):
        """Iterator methods."""
        return self._next_read()

    def _next_read(self):
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
            quality=rquality)

        for validator in self._validators:
            result, description = validator.validate(read)
            if not result or description:
                if self.lint_mode == "error":
                    raise SingleReadValidationError(
                        description=description,
                        readname=read.name,
                        filename=self.file.name,
                        lineno=self.file._lineno)
                elif self.lint_mode == "report":
                    print(
                        f"{self.file.basename}:{validator.name}:{self.file._lineno}: " \
                        f"{description}"
                    )
                else:
                    raise NotImplementedError(
                        f"Not implemented for lint mode: {self.lint_mode}")

        return read

    def close(self):
        """Closes the file handle."""
        self.file.handle.close()


class PairedFastQFiles:
    """A class that steps through two FastQFiles at the same time and validates them
    from a global perspective.
    
    Args:
        read_one_filename(str): Path to the read one file.
        read_two_filename(str): Path to the read two file.
        lint_mode(str): Linting in 'error' or 'report' mode.
        single_read_validation_level: Validation level for the single read errors. 
        paired_read_validation_level: Validation level for the paired read errors.
    """

    def __init__(self,
                 read_one_filename: str,
                 read_two_filename: str,
                 lint_mode="error",
                 single_read_validation_level="low",
                 paired_read_validation_level="low"):
        self.read_one_fastqfile = FastQFile(
            read_one_filename,
            validation_level=single_read_validation_level,
            lint_mode=lint_mode)
        self.read_two_fastqfile = FastQFile(
            read_two_filename,
            validation_level=single_read_validation_level,
            lint_mode=lint_mode)
        self.lint_mode = lint_mode
        self.vlevel = ValidationLevel.resolve(paired_read_validation_level)

        self._readno = 0
        self._validators = [
            v() for (k, v) in validators.__dict__.items()
            if not k.startswith("Base") and isinstance(v, type) and issubclass(
                v, BasePairedReadValidator) and v.level <= self.vlevel
        ]

    def __iter__(self):
        """Iterator methods."""
        return self

    def __next__(self):
        """Iterator methods."""
        result = self._next_readpair()
        self._readno += 1
        return result

    def _next_readpair(self):
        """Get the next pair of reads from both files.
        
        Returns:
            (FastQRead, FastQRead): tuple of both FastQRead objects, one from each file.

        Raises:
            Error: multiple errors may be thrown, especially FastQ validation errors
        """

        read_one = self.read_one_fastqfile._next_read()
        read_two = self.read_two_fastqfile._next_read()

        # Must be "and". If one is None and not the other, that's a mismatched FastQ
        # read in one of the files.
        if not read_one and not read_two:
            raise StopIteration

        for validator in self._validators:
            result, description = validator.validate(read_one, read_two)
            if not result or description:
                if self.lint_mode == "error":
                    raise PairedReadValidationError(
                        description=description,
                        read_one=read_one,
                        read_two=read_two,
                        read_pairno=self._readno,
                        read_one_fastqfile=self.read_one_fastqfile,
                        read_two_fastqfile=self.read_two_fastqfile)
                elif self.lint_mode == "report":
                    print(
                        f"{self.read_one_fastqfile.file.basename}:{validator.name}:" \
                        f"{self._readno}: {description}"
                    )
                    print(
                        f"{self.read_two_fastqfile.file.basename}:{validator.name}:" \
                        f"{self._readno}: {description}"
                    )
                else:
                    raise NotImplementedError(
                        f"Not implemented for lint mode: {self.lint_mode}")

        return (read_one, read_two)

    def get_validators(self):
        """Returns a tuple with (SingleReadValidators, PairedReadValidators)"""
        sr_validators = [(v.code, v.__class__.__name__)
                         for v in self.read_one_fastqfile._validators]
        pr_validators = [(v.code, v.__class__.__name__)
                         for v in self._validators]
        return (sr_validators, pr_validators)

    def close(self):
        """Close both FastQ files."""
        self.read_one_fastqfile.close()
        self.read_two_fastqfile.close()