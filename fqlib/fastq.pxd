cimport utils
from libc.stdio cimport *
from libcpp.string cimport string

# Lightweight struct for FastQRead
# --------------------------------
#
#     name (string): proper name of the read in the FastQ file.
#     sequence (string): sequence referred to in the read.
#     plusline (string): content of the 'plusline' in the read.
#     quality (string): corresponding quality of sequence in the read.
#     interleave (string): interleave of the read.

ctypedef struct FastQRead:
    string name
    string sequence
    string plusline
    string quality
    string interleave

cpdef str fqread_repr(FastQRead read)