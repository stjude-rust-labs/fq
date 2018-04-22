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

    read.name = name
    read.sequence = sequence
    read.plusline = plusline
    read.quality = quality

    # optional fields
    read.interleave = <char*> ""
    read.secondary_name = <char*> "" 

    # print(fqread_repr(read))

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


cdef void fqread_generate(FastQRead &read, char *interleave):
    """Generate values emulating an Illumina-based FastQ read."""

    cdef char *instrument = "illumina1"
    cdef char *run_number = "1"
    cdef char *flowcell = "AABBCC"
    cdef char *lane = "1"
    cdef char *tile = "1"
    cdef char *x_pos = "1"
    cdef char *y_pos = "1"
    cdef char *sequence = "AAAAAAAAAACCCCCCCCCGGGGGGGGGGTTTTTTTTTT"
    cdef char *plusline = "+"
    cdef char *quality =  "JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ"

    cdef char[1024] readname
    strcpy(readname, b"@")
    strcat(readname, instrument)
    strcat(readname, b":")
    strcat(readname, run_number)
    strcat(readname, b":")
    strcat(readname, flowcell)
    strcat(readname, b":")
    strcat(readname, lane)
    strcat(readname, b":")
    strcat(readname, tile)
    strcat(readname, b":")
    strcat(readname, x_pos)
    strcat(readname, b":")
    strcat(readname, y_pos)
    strcat(readname, interleave)

    fqread_init(
        read,
        readname,
        sequence,
        plusline,
        quality
    )

cdef void fqread_write_to_file(FastQRead &read, FILE *f):
    fputs(read.name, f)
    if strcmp(read.interleave, "") != 0:
        fputs(read.interleave, f)
    if strcmp(read.secondary_name, "") != 0:
        fputs(" ", f)
        fputs(read.secondary_name, f)
    fputs(b"\n", f)
    fputs(read.sequence, f)
    fputs(b"\n", f)
    fputs(read.plusline, f)
    fputs(b"\n", f)
    fputs(read.quality, f)
    fputs(b"\n", f)


cpdef str fqread_repr(FastQRead &read):
    cdef char[0x400] buff
    strcpy(buff, "FastQRead(name='")
    strcat(buff, read.name)
    strcat(buff, "', sequence='")
    strcat(buff, read.sequence)
    strcat(buff, "', plusline='")
    strcat(buff, read.plusline)
    strcat(buff, "', quality='")
    strcat(buff, read.quality)
    strcat(buff, "', interleave='")
    strcat(buff, read.interleave)
    strcat(buff, "', secondary_name='")
    strcat(buff, read.secondary_name)
    strcat(buff, "')")
    return buff