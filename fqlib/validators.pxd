# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++


from cpython cimport object
from fqlib.fqread cimport FastQRead
from libcpp cimport bool as cbool
from libcpp.string cimport string

cdef enum _ValidationLevel:
    MINIMUM = 1
    LOW = 2
    HIGH = 3

cdef class ValidationLevel:
    cdef public _ValidationLevel level

cdef class BaseSingleReadValidator:
    cdef public string error 
    cdef public string code
    cdef public ValidationLevel validation_level
    cpdef public cbool validate(self, FastQRead&)

cdef class BasePairedReadValidator:
    cdef public string error 
    cdef public string code
    cdef public ValidationLevel validation_level
    cpdef public cbool validate(self, FastQRead&, FastQRead&)