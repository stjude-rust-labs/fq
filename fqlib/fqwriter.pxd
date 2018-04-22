# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

from libc.stdio cimport FILE, fopen, fputs, fclose, sprintf
from libc.stdlib cimport rand
from fqlib.fqread cimport FastQRead, fqread_generate, fqread_write_to_file_add_interleave

cdef class FastQWriter:
    cdef public char* filename_readone
    cdef public char* filename_readtwo 
    cdef public char[64] instrument
    cdef public char[64] flowcell
    cdef public char[64] run_number

    cpdef generate(self, n_reads)