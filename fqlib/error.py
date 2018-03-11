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
                 pairno,
                 read_one_filename=None,
                 read_two_filename=None,
                 read_one_fastqfile=None,
                 read_two_fastqfile=None):
        self.description = description
        self.read_one = read_one
        self.read_two = read_two
        self.pairno = pairno
        self.read_one_filename = read_one_filename
        self.read_two_filename = read_two_filename
        self.read_one_fastqfile = read_one_fastqfile
        self.read_two_fastqfile = read_two_fastqfile

    def __str__(self):
        res = "Read Pair Number: {pairno}\n"
        res += "Read 1\n"
        if self.read_one_filename:
            res += "   - File: {read_one_filename}\n"

        if self.read_one_fastqfile._lineno:
            res += "   - Line Number: {read_one_fastqfile._lineno}\n"

        res += "   - Readname: {read_one.name}\n"
        res += "Read 2\n"

        if self.read_two_filename:
            res += "   - File: {read_two_filename}\n"

        if self.read_two_fastqfile._lineno:
            res += "   - Line Number: {read_two_fastqfile._lineno}\n"

        res += "   - Readname: {read_two.name}\n"
        res += "---\n"
        res += "{description}"

        return res.format(**self.__dict__)
