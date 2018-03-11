class SingleReadValidationError(Exception):
    def __init__(self, description, readname, filename=None, lineno=None):
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
    def __init__(self,
                 description,
                 read_one,
                 read_two,
                 read_pairno,
                 read_one_fastqfile=None,
                 read_two_fastqfile=None):
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
            res += "   - File: {read_one_fastqfile.file.basename}\n"
            res += "   - Line Number: {read_one_fastqfile.file._lineno}\n"

        res += "   - Readname: {read_one.name}\n"
        res += "Read 2\n"

        if self.read_two_fastqfile:
            res += "   - File: {read_two_fastqfile.file.basename}\n"
            res += "   - Line Number: {read_two_fastqfile.file._lineno}\n"

        res += "   - Readname: {read_two.name}\n"
        res += "---\n"
        res += "{description}"

        return res.format(**self.__dict__)
