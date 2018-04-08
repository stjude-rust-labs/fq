from libcpp cimport bool as cbool

cdef extern from "<string_view>" namespace "std" nogil:
    size_t npos = -1

    cdef cppclass string_view:
        string_view() except +
        string_view(const char*) except +

        data()

        starts_with(const char*) except +
        ends_with(const char*) except +

        substr(size_t, size_t)
        size()