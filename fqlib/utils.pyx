# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

cdef class CFileReader:
    """Utility class used internally to read files using the C API. This
    class is meant to be used primarily as a building block for the
    FastQFile python class."""

    def __cinit__(self, filename: str):
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

    def __dealloc__(self):
        if self.handle:
            fclose(self.handle)
            self.handle = NULL