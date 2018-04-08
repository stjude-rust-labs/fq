# cython: infertypes=True
# cython: language_level=3
# cython: c_string_type=unicode
# cython: c_string_encoding=ascii
# distutils: language=c++

DEF NUM_INTERLEAVES = 2
DEF INTERLEAVE_LEN = 2
cdef char *POSSIBLE_INTERLEAVES[NUM_INTERLEAVES]
POSSIBLE_INTERLEAVES[:] = [<char*> "/1",<char*> "/2"]

cdef void fqread_init(
    FastQRead &read, 
    char* name, 
    char* sequence, 
    char* plusline,
    char* quality
):
    """Initialize a FastQRead object based on the values passed in."""

    cdef char* interleave
    cdef char* suffix
    cdef char* tmp_name = NULL
    cdef char* tmp_secondary = NULL
    cdef int i = 0

    read.sequence = sequence
    read.plusline = plusline
    read.quality = quality

    # optional fields
    read.secondary_name = <char*> "" 
    read.interleave = <char*> ""

    # parse secondary name
    tmp_name = strtok(name, " ")
    tmp_secondary = strtok(NULL, "")

    if tmp_name != NULL:
        read.name = tmp_name 
    if tmp_secondary != NULL:
        read.secondary_name = tmp_secondary

    # parse interleave
    cdef size_t name_len = strlen(read.name)
    cdef size_t suffix_offset = name_len - INTERLEAVE_LEN
    while i < NUM_INTERLEAVES:
        interleave = POSSIBLE_INTERLEAVES[i]
        if strcmp(read.name + suffix_offset, interleave) == 0:
            read.name[suffix_offset] = b'\0'
            read.interleave = interleave
        i += 1


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
        "@" + instrument + ":" + run_number + ":" + flowcell + ":" + lane + ":" + tile + ":" + x_pos + ":" + y_pos,
        sequence,
        plusline,
        quality
    )

cpdef FastQRead fqread_generate_new():
    cdef FastQRead read
    fqread_generate(read)
    return read

cpdef str fqread_repr(FastQRead &read):
   return f"FastQRead(name='{read.name)}', "\
           f"sequence='{read.sequence}', " \
           f"plusline='{read.plusline}', " \
           f"quality='{read.quality}', " \
           f"interleave='{read.interleave}', " \
           f"secondary_name='{read.secondary_name}')"