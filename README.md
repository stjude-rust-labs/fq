# fqlib

[![CI status](https://github.com/stjude/fqlib/workflows/CI/badge.svg)](https://github.com/stjude/fqlib/actions)

**fqlib** is a library to generate and validate [FASTQ] file pairs.

[FASTQ]: https://en.wikipedia.org/wiki/FASTQ_format

## Install

There are different methods to install fqlib. The binary is named `fq`.

### Releases

[Precompiled binaries are built][releases] for modern Linux distributions
(`x86_64-unknown-linux-gnu`), macOS (`x86_64-apple-darwin`), and Windows
(`x86_64-pc-windows-msvc`). The Linux binaries require glibc 2.18+ (CentOS/RHEL
8+, Debian 8+, Ubuntu 14.04+, etc.).

[releases]: https://github.com/stjude/fqlib/releases

### Manual

Clone the repository and use [Cargo] to install fqlib.

```
$ git clone --depth 1 --branch v0.7.1 https://github.com/stjude/fqlib.git
$ cd fqlib
$ cargo install --path .
```

[Cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html

### Container image

Alternatively, build the container image, e.g., using [Docker]:

```
$ git clone --depth 1 --branch v0.7.1 https://github.com/stjude/fqlib.git
$ cd fqlib
$ docker image build --tag fqlib:0.7.1 .
```

[Docker]: https://www.docker.com/

## Usage

fqlib provides subcommands for filtering, generating, and validating FASTQ
files.

### filter

**fq filter** takes an allowlist of record names and filters a given FASTQ
file. The result includes only the records in the allowlist.

#### Usage

```
fq-filter
Filters a FASTQ from an allowlist of names

USAGE:
    fq filter <src> --names <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --names <path>    Allowlist of record names

ARGS:
    <src>    Source FASTQ
```

#### Examples

```sh
# Filters an input FASTQ using the given allowlist.
$ fq filter --names allowlist.txt in.fastq
```

### generate

**fq generate** is a FASTQ file pair generator. It creates two reads, formatting
names as [described by Illumina][1].

While _generate_ creates "valid" FASTQ reads, the content of the files are
completely random. The sequences do not align to any genome.

[1]: https://help.basespace.illumina.com/articles/descriptive/fastq-files/

#### Usage

```
fq-generate
Generates a random FASTQ file pair

USAGE:
    fq generate [OPTIONS] <r1-dst> <r2-dst>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --read-length <usize>    Number of bases in the sequence [default: 101]
    -n, --record-count <u64>     Number of records to generate [default: 10000]
        --seed <u64>             Seed to use for the random number generator

ARGS:
    <r1-dst>    Read 1 destination. Output will be gzipped if ends in `.gz`.
    <r2-dst>    Read 2 destination. Output will be gzipped if ends in `.gz`.
```

#### Examples

```sh
# Generates the default number of records, written to uncompressed files.
$ fq generate /tmp/r1.fastq /tmp/r2.fastq

# Generates FASTQ paired reads with 32 records, written to gzipped outputs.
$ fq generate --record-count 32 /tmp/r1.fastq.gz /tmp/r2.fastq.gz
```

### lint

**fq lint** is a FASTQ file pair validator.

#### Usage

```
fq-lint
Validates a FASTQ file pair

USAGE:
    fq lint [OPTIONS] <r1-src> [--] [r2-src]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --disable-validator <str>...            Disable validators by code. Use multiple times to disable more than one.
        --lint-mode <str>                       Panic on first error or log all errors. Logging forces verbose mode.
                                                [default: panic]  [possible values: panic, log]
        --paired-read-validation-level <str>    Only use paired read validators up to a given level [default: high]
                                                [possible values: low, medium, high]
        --single-read-validation-level <str>    Only use single read validators up to a given level [default: high]
                                                [possible values: low, medium, high]

ARGS:
    <r1-src>    Read 1 source. Accepts both raw and gzipped FASTQ inputs.
    <r2-src>    Read 2 source. Accepts both raw and gzipped FASTQ inputs.
```

#### Validators

_validate_ includes a set of validators that run on single or paired records.
By default, records are validated with all rules, but validators can be
disabled using `--disable-valdiator CODE`, where `CODE` is one of validators
listed below.

##### Single

| Code | Level  | Name              | Validation
|------|--------|-------------------|------------
| S001 | low    | PlusLine          | Plus line starts with a "+".
| S002 | medium | Alphabet          | All characters in sequence line are one of "ACGTN", case-insensitive.
| S003 | high   | Name              | Name line starts with an "@".
| S004 | low    | Complete          | All four record lines (name, sequence, plus line, and quality) are present.
| S005 | high   | ConsistentSeqQual | Sequence and quality lengths are the same.
| S006 | medium | QualityString     | All characters in quality line are between "!" and "~" (ordinal values).
| S007 | high   | DuplicateName     | All record names are unique.

##### Paired

| Code | Level   | Name              | Validation
|------|---------|-------------------|------------
| P001 | medium  | Names             | Each paired read name is the same, excluding interleave.

#### Examples

```sh
# Validate both reads using all validators. Exits cleanly (0) if no validation
# errors occur.
$ fq lint r1.fastq r2.fastq

# Enable log messages.
$ fq --verbose lint r1.fastq r2.fastq

# Log errors instead of quitting on first error.
$ fq lint --lint-mode log r1.fastq r2.fastq

# Disable validators S004 and S007.
$ fq lint --disable-validator S004 --disable-validator S007 r1.fastq r2.fastq
```
