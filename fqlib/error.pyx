class SingleReadValidationError(Exception):
    """Validation error resulting from a malformed single read."""

    def __init__(self, description, readname, filename=None, lineno=None):
        super().__init__()
        self.description = description
        self.readname = readname
        self.filename = filename
        self.lineno = lineno

    def __str__(self):
        res = ""
        if self.filename:
            res += "Filename: {filename}\n"
        if self.lineno:
            res += "Line Number: {lineno}\n"
        res += "Readname: {readname}\n"
        res += "---\n"
        res += "{description}"

        return res.format(**self.__dict__)


class PairedReadValidationError(Exception):
    """Validation error resulting from a malformed pair of reads."""

    def __init__(
        self,
        description,
        read_one,
        read_two,
        read_pairno,
        read_one_fastqfile=None,
        read_two_fastqfile=None
    ):
        super().__init__()
        self.description = description
        self.read_one = read_one
        self.read_two = read_two
        self.read_pairno = read_pairno
        self.read_one_fastqfile = read_one_fastqfile
        self.read_two_fastqfile = read_two_fastqfile

    def __str__(self):
        res = "Read Pair Number: {read_pairno}\n"
        res += "Read 1\n"
        if self.read_one_fastqfile:
            res += "   - File: {read_one_fastqfile.basename}\n"
            res += "   - Line Number: {read_one_fastqfile.cfile_handle.lineno}\n"

        res += f"   - Readname: {self.read_one['name']}\n"
        res += "Read 2\n"

        if self.read_two_fastqfile:
            res += "   - File: {read_two_fastqfile.basename}\n"
            res += "   - Line Number: {read_two_fastqfile.cfile_handle.lineno}\n"

        res += f"   - Readname: {self.read_two['name']}\n"
        res += "---\n"
        res += "{description}"

        return res.format(**self.__dict__)
