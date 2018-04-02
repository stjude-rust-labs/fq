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
        if hasattr(self, 'filename'):
            res += f"Filename: {self.filename}\n"
        if hasattr(self, 'lineno'):
            res += f"Line Number: {self.lineno}\n"
        res += f"Readname: {self.readname}\n"
        res += "---\n"
        res += f"{self.description}"
        return res


class PairedReadValidationError(Exception):
    """Validation error resulting from a malformed pair of reads."""

    def __init__(
        self,
        description,
        read_one,
        read_two,
        read_pairno,
        read_one_fastqfile,
        read_two_fastqfile,
    ):
        super().__init__()
        self.description = description
        self.read_one = read_one
        self.read_two = read_two
        self.read_pairno = read_pairno
        self.read_one_fastqfile = read_one_fastqfile
        self.read_two_fastqfile = read_two_fastqfile

    def __str__(self):
        res =  f"Read Pair Number: {self.read_pairno}\n"
        res += "Read 1\n"
        res += f"   - File: {self.read_one_fastqfile.basename}\n"
        res += f"   - Line Number: {self.read_one_fastqfile.cfile_handle.lineno}\n"
        res += f"   - Readname: {self.read_one.get('name')}\n"
        res += "Read 2\n"
        res += f"   - File: {self.read_two_fastqfile.basename}\n"
        res += f"   - Line Number: {self.read_two_fastqfile.cfile_handle.lineno}\n"
        res += f"   - Readname: {self.read_two.get('name')}\n"
        res += "---\n"
        res += f"{self.description}"
        return res
