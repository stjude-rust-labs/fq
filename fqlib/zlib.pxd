cdef extern from *:
    ctypedef char const_char "const char"

cdef extern from "zlib.h":
    ctypedef void* gzFile

    int Z_NULL

    gzFile gzopen(const_char *path, const_char *mode)
    int gzread(gzFile f, void *buff, unsigned long length)
    int gzeof(gzFile f)
    int gzclose(gzFile f)

