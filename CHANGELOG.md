# Changelog

## 0.9.1 - 2022-02-15

### Fixed

  * commands/subsample: Count the lines from the decompressed data if the input
    is gzipped.

    Used in the exact sampler, this previously counted "lines" from the
    compressed input.

  * commands/subsample: Clamp the destination record count to the range of the
    source record count.

    Otherwise, this would cause the filter to never finish building.

## 0.9.0 - 2022-02-10

### Added

  * commands/subsample: Add exact sampler.

    This writes an exact number of samples to the output. Set the
    `-n/--record-count` option to use the exact sampler.

### Changed

  * Update argument parser to clap 3.

## 0.8.0 - 2021-11-12

### Changed

  * Rename project to fq.

### Added

  * commands/generate: Add `-s` short option for `--seed`.

  * commands: Add `subsample` command.

    `subsample` outputs a proportional subset of records from single or paired
    FASTQ files.

### Deprecated

  * Deprecate `--verbose` flag.

    Logging is now always enabled.

## 0.7.1 - 2021-10-07

### Fixed

  * main: Show global version in subcommands ([#20]).

    This allows subcommands to show the global version, e.g., `fq lint
    --version`.

[#20]: https://github.com/stjude-rust-labs/fq/issues/20

## 0.7.0 - 2021-05-07

### Added

  * `generate`: Added `--read-length` option to set the number of bases to
    generate in each record's sequence.

## 0.6.0 - 2020-12-17

### Added

  * The FASTQ reader handles files with CRLF (Windows) newlines and no final
    newline.

## 0.5.0 - 2020-10-09

### Changed

  * [BREAKING] `generate`: Renamed `--n-records` to `--record-count`.

  * `generate`: `--record-count` is parsed as a `u64` rather than an `i32`. The
    argument parser never allowed negative numbers, so this change still
    includes the entire previous input set.

## 0.4.0 - 2020-06-30

### Added

  * The `generate` command adds a `--seed <u64>` option to seed the random
    number generator. This is useful to regenerate the same outputs.

### Changed

  * The FASTQ generator now uses the Sanger/Illumina 1.8+ range of
    quality scores ([0, 41]). It samples scores on a normal distribution (μ =
    20.5, σ = 2.61).

## 0.3.1 - 2019-08-14

### Changed

  * Updated dependency `bloom` --> `bbloom` to reflect a name change in the library.

## 0.3.0 - 2019-08-09

### Added

  * New `filter` command. This accepts an allowlist of record read names to
    keep in the output FASTQ.

  * Add `Dockerfile` to build a self-contained image for `fq`. Build with
    `docker build --tag fqlib .`.

  * Show git commit ID and date in display version, e.g., when using
    `--version`. This makes it easier to know the exact build of fqlib being
    used.

### Changed

  * [BREAKING] `generate`: Renamed `--num-blocks` to `--n-records`.

## 0.2.0 - 2018-11-28

### Added

  * For paired end reads, `fq lint` exits with unexpected EOF if the both
    streams do not finish together.

  * Multistream gzip files can be used as inputs. Written files still use a
    single stream.

  * `fq lint` can take one FASTQ file as input for only single read validation.

### Changed

  * A single binary `fq` with subcommands replaces `fqgen` and `fqlint`. Update
    usages to `fq generate` and `fq lint`, respectively.

  * Metadata from CASAVA 1.8 read names is truncated. This is handled the same
    as interleaves.

### Fixed

  * Fix line offset in error messages, which was previously off by 4.

## 0.1.0 - 2018-06-05

  * Initial release
