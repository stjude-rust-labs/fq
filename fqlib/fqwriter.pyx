# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

cdef class FastQWriter:

    def __cinit__(self, filename_readone, filename_readtwo):
        self.filename_readone = filename_readone
        self.filename_readtwo = filename_readtwo
        sprintf(self.instrument, "fqlib%d", <int> (rand() % 10 + 1))
        sprintf(self.flowcell, "AABBCC")
        sprintf(self.run_number, "%d", <int> (rand() % 1000 + 1))

    cpdef generate(self, n_reads):
        cdef FastQRead read
        cdef FILE *f_readone
        cdef FILE *f_readtwo

        f_readone = fopen(self.filename_readone, "w")
        if f_readone == NULL:
            raise RuntimeError("Could not open R1 file!")

        f_readtwo = fopen(self.filename_readtwo, "w")
        if f_readtwo == NULL:
            raise RuntimeError("Could not open R2 file!")

        for i in range(n_reads):
            fqread_generate(read, self.instrument, self.flowcell, self.run_number, "")
            fqread_write_to_file_add_interleave(read, f_readone, "/1")
            fqread_write_to_file_add_interleave(read, f_readtwo, "/2")

        fclose(f_readone)
        fclose(f_readtwo)