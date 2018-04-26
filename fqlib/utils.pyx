# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

DEF CHAR_LIMIT_MAX_PER_LINE = 0x8000 # also needs to be updated in utils.pxd

cdef class CFileReader:
    """Utility class used internally to read files using the C API. This
    class is meant to be used primarily as a building block for the
    SingleFastQReader python class."""

    def __cinit__(self, filename: str):
        self.filename = realpath(filename, NULL)
        self.handle = gzopen(self.filename, "r")
        if self.handle == NULL:
            raise RuntimeError("Could not open file!")
        self.lineno = 0
        self.buffer_len = CHAR_LIMIT_MAX_PER_LINE
        self.nread = -1

    cdef ssize_t getline_clip(self):
        """Processes a line from a file, reading one character at a time and handling
        accordingly.

        TODO: Opportunity to optimize this better by *not* reading one character at a
              time (what a novel idea).
        """
        cdef int index = 0
        cdef char b
        cdef ssize_t nread = -1

        # read first char, if -1, there is nothing left in the stream.
        nread = gzread(self.handle, &b, 1) 
        if nread == -1:
            return -1

        while gzeof(self.handle) != 1:
            if b == 13: # Ignore carriage returns
                pass 
            elif b == 10: # The line ends with a newline, so exit loop
                break 
            else:
                self.buffer[index] = b
                index +=1
                if index >= self.buffer_len:
                    break

            nread = gzread(self.handle, &b, 1)
 
        self.buffer[index] = b'\0'
        return index

    cdef char* read_line(self):
        """Reads a line from a file, advances the line number if needed."""
        self.nread = self.getline_clip()

        if self.nread < 0:
            return <char*> ""

        self.lineno = self.lineno + 1
        return self.buffer 

    def __dealloc__(self):
        free(self.filename)
        if self.handle:
            gzclose(self.handle)
            self.handle = NULL
