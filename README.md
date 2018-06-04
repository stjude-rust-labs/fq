# fqlib

**fqlib** is a library to generate and validate [FASTQ] file pairs.

[FASTQ]: https://en.wikipedia.org/wiki/FASTQ_format

## Install

Use [Cargo] to install the two command-line tools included with fqlib:
**fqgen** and **fqlint**.

```
$ cargo install --git https://github.com/zaeleus/fqlib.git
```

[Cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html

## fqgen

**fqgen** is a FASTQ file pair generator. It creates two reads, formatting
names as [described by Illumina][1].

While fqgen will generate "valid" FASTQ reads, the content of the files are
completely random. The sequences will not align to any genome.

[1]: https://help.basespace.illumina.com/articles/descriptive/fastq-files/

### Usage

```
$ fqgen --help

fqgen 0.1.0

USAGE:
    fqgen [FLAGS] [OPTIONS] <out1> <out2>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Use verbose logging

OPTIONS:
    -n, --num-blocks <N>    Number of blocks to generate [default: 10000]

ARGS:
    <out1>    Read 1 output pathname. Output will be gzipped if ends in `.gz`.
    <out2>    Read 2 output pathname. Output will be gzipped if ends in `.gz`.
```

### Examples

```sh
# Generates the default number of blocks, written to uncompressed files.
$ fqgen /tmp/r1.fastq /tmp/r2.fastq

# Generates FASTQ paired reads with 32 blocks, written to gzipped outputs.
$ fqgen --num-blocks 32 /tmp/r1.fastq.gz /tmp/r2.fastq.gz
```

## fqlint

**fqlint** is a FASTQ file pair validator.

### Usage

```
$ fqlint --help

USAGE:
    fqlint [FLAGS] [OPTIONS] <in1> <in2>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Use verbose logging

OPTIONS:
        --disable-validator <CODE>...
            Disable validators by code. Use multiple times to disable more than one.

        --lint-mode <MODE>
            Panic on first error or log all errors. Logging forces verbose mode. [default: panic]  [possible values:
            panic, log]
        --paired-read-validation-level <LEVEL>
            Only use paired read validators up to a given level [default: high]  [possible values: low, medium, high]

        --single-read-validation-level <LEVEL>
            Only use single read validators up to a given level [default: high]  [possible values: low, medium, high]


ARGS:
    <in1>    Read 1 input pathname. Accepts both raw and gzipped FASTQ inputs.
    <in2>    Read 2 input pathname. Accepts both raw and gzipped FASTQ inputs.
```

### Validators

fqlint includes a set of validators that run on single blocks or block pairs.
By default, blocks will be validated with all rules, but validators can be
disabled using `--disable-valdiator CODE`, where `CODE` is one of validators
listed below.

#### Single

| Code | Level  | Name              | Validation
|------|--------|-------------------|------------
| S001 | low    | PlusLine          | Plus line starts with a "+".
| S002 | medium | Alphabet          | All characters in sequence line are one of "ACGTN", case-insensitive.
| S003 | high   | Name              | Name line starts with an "@".
| S004 | low    | Complete          | All four block lines (name, sequence, plus line, and quality) are present.
| S005 | high   | ConsistentSeqQual | Sequence and quality lengths are the same.
| S006 | medium | QualityString     | All characters in quality line are between "!" and "~" (ordinal values).
| S007 | high   | DuplicateName     | All block names are unique.

#### Paired

| Code | Level   | Name              | Validation
|------|---------|-------------------|------------
| P001 | medium  | Names             | Each paired read name is the same, excluding interleave.

### Examples

```sh
# Validate both reads using all validators. Exits cleanly (0) if no validation
# errors occur.
$ fqlint r1.fastq r2.fastq

# Enable log messages.
$ fqlint --verbose r1.fastq r2.fastq

# Log errors instead of quitting on first error.
$ fqlint --lint-mode log r1.fastq r2.fastq

# Disable validators S004 and S007.
$ fqlint --disable-validator S004 --disable-validator S007 r1.fastq r2.fastq
```
