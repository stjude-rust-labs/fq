cdef class FastQWriter:

    cdef public str filename

    def __cinit__(self, filename):
        self.filename = filename

    def generate(n_reads):
        print(n_reads)