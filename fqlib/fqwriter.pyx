# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

cdef class FastQWriter:

    def __cinit__(self, filename, interleave = ""):
        self.filename = filename
        self.interleave = interleave

    cpdef generate(self, n_reads):
        cdef FastQRead read
        cdef FILE *f

        f = fopen(self.filename, "w")
        if f == NULL:
            raise RuntimeError("Could not open file!")

        for i in range(n_reads):
            fqread_generate(read, self.interleave)
            fqread_write_to_file(read, f)

        fclose(f)