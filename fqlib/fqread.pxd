# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from libcpp cimport bool as cbool
from libc.stdio cimport FILE, fopen, fclose, fputs
from libc.string cimport strtok, strcmp, strrchr, strlen, strcat, strcpy

ctypedef struct FastQRead:
    # required fields
    char* name
    char* sequence
    char* plusline
    char* quality

    # optional fields
    char* secondary_name
    char* interleave

cdef void fqread_init(FastQRead&, char* name, char* sequence, char* plusline, char* quality)
cdef void fqread_write_to_file(FastQRead &read, FILE *f)
cdef void fqread_generate(FastQRead& read, char* interleave)
cpdef str fqread_repr(FastQRead &read)
