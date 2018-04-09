# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from libc.stdio cimport *
from libcpp cimport bool as cbool
from libcpp.vector cimport vector

cdef class CFileReader:

    # private attributes
    cdef char* filename
    cdef FILE* handle
    cdef size_t rlen
    cdef ssize_t nread
    cdef char* line

    # read-only attributes
    cdef readonly int lineno

    # methods
    cdef char* read_line(self)
