# Changelog

## [Unreleased]

### Added

  * New `filter` command. This accepts a whitelist of record read names to keep
    in the output FASTQ.

  * Add `Dockerfile` to build a self-contained image for `fq`. Build with
    `docker build --tag fqlib .`.

  * Show git commit ID and date in display version, e.g., when using
    `--version`. This makes it easier to know the exact build of fqlib being
    used.

## [0.2.0] - 2018-11-28

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

[Unreleased]: https://github.com/stjude/fqlib/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/stjude/fqlib/compare/v0.1.0...v0.2.0
