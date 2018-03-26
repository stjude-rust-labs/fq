# cython: infertypes=True, language_level=3
# distutils: language=c++

DEF READ_ENCODING = "utf-8"

from libcpp.string cimport string

cdef class CFileReader:
    """Utility class used internally to read files using the C API. This
    class is meant to be used primarily as a building block for the 
    FastQFile python class."""

    def __init__(self, filename: str):
        cdef bytes fn_bytes = filename.encode(READ_ENCODING)
        self.filename = fn_bytes
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