# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from cpython cimport object
from libc.stdio cimport *
from libcpp cimport bool as cbool
from libcpp.string cimport string

cdef extern from "<algorithm>" namespace "std" nogil:
    cbool equal[Iter1,Iter2] (Iter1 first1, Iter1 last1, Iter2 first2)

cdef inline cbool ends_with(string &value, string &ending):
    if (ending.size() > value.size()): return False
    return equal(ending.rbegin(), ending.rend(), value.rbegin())

ctypedef struct FastQRead:
    string name
    string sequence
    string plusline
    string quality
    string interleave

cpdef FastQRead fqread_generate()
cpdef str fqread_repr(FastQRead read)
cpdef FastQRead fqread_init(string name, string sequence, string plusline, string quality)

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
    cdef string read_line(self)
    cdef void close(self)
