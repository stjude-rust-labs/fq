# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from fqlib.utils cimport ends_with
from libcpp.string cimport string

ctypedef struct FastQRead:
    string name
    string sequence
    string plusline
    string quality
    string interleave

cdef void fqread_init(FastQRead&, string, string, string, string)
cdef void fqread_generate(FastQRead&)
#cpdef str fqread_repr(FastQRead &read)
