import os
from collections import namedtuple

from . import validators
from .error import SingleReadValidationError, PairedReadValidationError
ValidationLevel = validators.ValidationLevel
BaseSingleReadValidator = validators.BaseSingleReadValidator
BasePairedReadValidator = validators.BasePairedReadValidator

class FastQRead:
    __slots__ = ['name', 'sequence', 'plusline', 'quality', 'interleave']

    def __init__(self, name, sequence, plusline, quality, interleave):
        self.name = name
        self.sequence = sequence
        self.plusline = plusline
        self.quality = quality
        self.interleave = interleave

    def __repr__(self):
        return f"FastQRead(name={self.name}, sequence={self.sequence}, " \
               f"plusline={self.plusline}, quality={self.quality}, " \
               f"interleave={self.interleave})"


class FastQFile:
    """Class used to iterate over records in a FastQ file. Validation will be
    performed based on the level of validation set in the constructor. Eventually,
    we will add functionality to efficiently unpack only the needed parts of the
    FastQ file.

    Args:
        filename(str): Path to the FastQ file.
        validation_level(validators.ValidationLevel): Degree to which the FastQ file
                       should be validated.

    Raises:
        FileNotFoundError, ValueError
    """

    def __init__(self,
                 filename: str,
                 validation_level: ValidationLevel=ValidationLevel.HIGH):
        self._filename = os.path.abspath(filename)
        self._handle = open(self._filename, 'r')
        self._lineno = 0

        if isinstance(validation_level, ValidationLevel):
            self._validation_level = validation_level
        elif isinstance(validation_level, str):
            if validation_level.lower() == "high":
                self._validation_level = ValidationLevel.HIGH
            elif validation_level.lower() == 'low':
                self._validation_level = ValidationLevel.LOW
            elif validation_level.lower() == 'minimum' or not validation_level:
                self._validation_level = ValidationLevel.MINIMUM

        if not self._validation_level:
            raise ValueError(f"Unknown single read validation level: {validation_level}")

        single_read_validators = [v() for (k, v) in validators.__dict__.items() if
                                  not k.startswith("Base") and isinstance(v, type)
                                  and issubclass(v, BaseSingleReadValidator)]
    
        self._validators = [v for v in single_read_validators if 
                            v.level >= self._validation_level]


    def __iter__(self):
        """Iterator methods."""
        return self
 

    def __next__(self):
        """Iterator methods."""
        return self._next_read()


    def _next_line(self):
        """Read the next line from the file handle."""
        result = self._handle.readline().strip()
        self._lineno += 1
        return result


    def _next_read(self):
        """Naively read the FastQ read. If something goes awry, expect it to get caught
        in the validators.
        
        Returns:
            FastQRead: a read from the FastQ file
            
        Raises:
            Error: multiple errors may be thrown, especially FastQ validation errors.
        """

        read_interleave = None
        read_name = self._next_line()
        interleaves = ["/1", "/2"]
        for interleave in interleaves:
            if read_name.endswith(interleave):
                read_name = read_name[:-len(interleave)]
                read_interleave = interleave
        read_sequence = self._next_line()
        read_plusline = self._next_line()
        read_quality = self._next_line()

        if not read_name or not read_sequence or not read_plusline or not read_quality:
            raise StopIteration

        read = FastQRead(name=read_name,
                         sequence=read_sequence,
                         plusline=read_plusline,
                         quality=read_quality, 
                         interleave=read_interleave)

        for validator in self._validators:
            result, err = validator.validate(read)
            if not result or err:
                raise SingleReadValidationError(description=err,
                                                readname=read.name,
                                                filename=self._filename,
                                                lineno=self._lineno)
        return read


    def close(self):
        """Closes the file handle."""
        self._handle.close()


class PairedFastQFiles:
    """A class that steps through two FastQFiles at the same time and validates them
    from a global perspective.
    
    Args:
        read_one_filename(str): Path to the read one file.
        read_two_filename(str): Path to the read two file.
        single_read_validation_level: Validation level for the single read errors. 
        paired_read_validation_level: Validation level for the paired read errors.
    """

    def __init__(self, read_one_filename: str, read_two_filename: str,
        single_read_validation_level="low", paired_read_validation_level="low"):

        self._read_one_filename = os.path.abspath(read_one_filename)
        self._read_two_filename = os.path.abspath(read_two_filename)
        self._read_one_fastqfile = FastQFile(self._read_one_filename,
                                            validation_level=single_read_validation_level)
        self._read_two_fastqfile = FastQFile(self._read_two_filename,
                                            validation_level=single_read_validation_level)
        self._readno = 0
        
        if isinstance(paired_read_validation_level, ValidationLevel):
            self._validation_level = paired_read_validation_level
        elif isinstance(paired_read_validation_level, str):
            if paired_read_validation_level.lower() == "high":
                self._validation_level = ValidationLevel.HIGH
            elif paired_read_validation_level.lower() == 'low':
                self._validation_level = ValidationLevel.LOW
            elif paired_read_validation_level.lower() == 'minimum' or not paired_read_validation_level:
                self._validation_level = ValidationLevel.MINIMUM

        if not self._validation_level:
            raise ValueError(f"Unknown single read validation level: {paired_read_validation_level}")
        
        paired_read_validators = [v() for (k, v) in validators.__dict__.items() if
                                  not k.startswith("Base") and isinstance(v, type)
                                  and issubclass(v, BasePairedReadValidator)]

        self._validators = [v for v in paired_read_validators if 
                            v.level >= self._validation_level]


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
            
        read_one = self._read_one_fastqfile._next_read()
        read_two = self._read_two_fastqfile._next_read()

        if not read_one or not read_two:
            raise StopIteration

        for validator in self._validators:
            result, err = validator.validate(read_one, read_two)
            if not result or err:
                raise PairedReadValidationError(description=err,
                                                read_one=read_one,
                                                read_two=read_two,
                                                pairno=self._readno,
                                                read_one_filename=self._read_one_filename,
                                                read_two_filename=self._read_two_filename,
                                                read_one_fastqfile=self._read_one_fastqfile,
                                                read_two_fastqfile=self._read_two_fastqfile)

        return (read_one, read_two)

            
    def close(self):
        """Close both FastQ files."""
        self._read_one_fastqfile.close()
        self._read_two_fastqfile.close()