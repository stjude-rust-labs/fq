# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from cpython cimport object
from libc.string cimport strcmp, strcpy
from libcpp.string cimport string
from fqlib.fqread cimport (
    FastQRead,
    fqread_init,
    fqread_repr
)
from fqlib.utils cimport CFileReader