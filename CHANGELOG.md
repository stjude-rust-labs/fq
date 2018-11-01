# Changelog

## Unreleased

### Added

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
