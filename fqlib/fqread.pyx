# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

cdef string[2] POSSIBLE_INTERLEAVES
POSSIBLE_INTERLEAVES[:] = [<char*> "/1",<char*> "/2"]

cdef void fqread_init(FastQRead &read, string name, string sequence, string plusline,
    string quality):
    """Initialize a FastQRead object based on the values passed in."""

    read.name = name
    read.sequence = sequence
    read.plusline = plusline
    read.quality = quality
    read.interleave = <char*> ""

    cdef int i = 0

    for i in range(len(POSSIBLE_INTERLEAVES)):
        interleave = POSSIBLE_INTERLEAVES[i]
        if ends_with(read.name, interleave):
            read.name = read.name.substr(0, read.name.size() - 2)
            read.interleave = read.name.substr(read.name.size() - 2, 2)


cdef void fqread_generate(FastQRead &read):
    """Generate values emulating an Illumina-based FastQ read."""

    cdef instrument = "illumina1"
    cdef run_number = "1"
    cdef flowcell = "AABBCC"
    cdef lane = "1"
    cdef tile = "1"
    cdef x_pos = "1"
    cdef y_pos = "1"
    cdef sequence = "AAAAAAAAAACCCCCCCCCGGGGGGGGGGTTTTTTTTTT"
    cdef plusline = "+"
    cdef quality =  "JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ"

    fqread_init(
        read,
        instrument + ":" + run_number + ":" + flowcell + ":" + lane + ":" + tile + ":" + x_pos + ":" + y_pos,
        sequence,
        plusline,
        quality
    )

#cpdef str fqread_repr(FastQRead read):
#    return f"FastQRead(name='{read.name)}', "\
#            f"sequence='{read.sequence}', " \
#            f"plusline='{read.plusline}', " \
#            f"quality='{read.quality}', " \
#            f"interleave='{read.interleave}')"