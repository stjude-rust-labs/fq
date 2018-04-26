# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

DEF CHAR_LIMIT_MAX_PER_LINE = 0x8000 # also needs to be updated in utils.pyx

from libc.stdio cimport *
from libc.stdlib cimport free
from posix.stdlib cimport realpath
from libc.string cimport strchr, memset, strncpy
from libcpp cimport bool as cbool
from libcpp.vector cimport vector
from fqlib.zlib cimport (
    gzFile, 
    gzopen, 
    gzread, 
    gzeof,
    gzclose, 
    Z_NULL
)

cdef class CFileReader:

    # private attributes
    cdef char* filename
    cdef gzFile handle
    cdef char buffer[CHAR_LIMIT_MAX_PER_LINE]
    cdef ssize_t nread
    cdef size_t buffer_len

    # read-only attributes
    cdef readonly int lineno

    # methods
    cdef ssize_t getline_clip(self)
    cdef char* read_line(self)
