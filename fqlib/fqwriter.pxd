# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from libc.stdio cimport FILE, fopen, fputs, fclose
from fqlib.fqread cimport FastQRead, fqread_generate, fqread_write_to_file

cdef class FastQWriter:
    cdef public char* filename
    cdef public char* interleave 

    cpdef generate(self, n_reads)