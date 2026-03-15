use std::{path::PathBuf, str::FromStr};

use clap::{ArgGroup, Parser, Subcommand};
use git_testament::{git_testament, render_testament};
use regex::bytes::Regex;

use crate::{ValidationLevel, validators::LintMode};

git_testament!(TESTAMENT);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AsciiChar(u8);

impl From<AsciiChar> for u8 {
    fn from(c: AsciiChar) -> Self {
        c.0
    }
}

impl FromStr for AsciiChar {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().next() {
            if let Ok(b) = u8::try_from(c) {
                Ok(Self(b))
            } else {
                Err("invalid character found in string")
            }
        } else {
            Err("cannot parse character from empty string")
        }
    }
}

#[derive(Parser)]
#[command(propagate_version = true, version = render_testament!(TESTAMENT))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Collect FASTQ metrics.
    Describe(DescribeArgs),
    /// Filters a FASTQ file.
    Filter(FilterArgs),
    /// Validates a FASTQ file pair.
    Lint(LintArgs),
    /// Outputs a subset of records.
    Subsample(SubsampleArgs),
}

#[derive(Parser)]
pub struct DescribeArgs {
    /// FASTQ source.
    pub src: PathBuf,
}

#[derive(Parser)]
#[command(group(ArgGroup::new("filter").args(["names", "sequence_pattern"])))]
pub struct FilterArgs {
    /// Allowlist of record names.
    #[arg(long)]
    pub names: Option<PathBuf>,

    /// Keep records that have sequences that match the given regular expression.
    #[arg(long)]
    pub sequence_pattern: Option<Regex>,

    /// Filtered FASTQ destinations.
    #[arg(long, required = true)]
    pub dsts: Vec<PathBuf>,

    /// FASTQ sources. Accepts both raw and gzipped FASTQ inputs.
    pub srcs: Vec<PathBuf>,
}

#[derive(Parser)]
pub struct LintArgs {
    /// Panic on first error or log all errors.
    #[arg(long, value_enum, default_value_t = LintMode::Panic)]
    pub lint_mode: LintMode,

    /// Only use single read validators up to a given level.
    #[arg(long, value_enum, default_value_t = ValidationLevel::High)]
    pub single_read_validation_level: ValidationLevel,

    /// Only use paired read validators up to a given level.
    #[arg(long, value_enum, default_value_t = ValidationLevel::High)]
    pub paired_read_validation_level: ValidationLevel,

    /// Disable validators by code. Use multiple times to disable more than one.
    #[arg(long)]
    pub disable_validator: Vec<String>,

    /// Define a record definition separator.
    ///
    /// This is used to strip the description from a record name.
    ///
    /// [default: '/' and ' ']
    #[arg(long)]
    pub record_definition_separator: Option<AsciiChar>,

    /// Read 1 source. Accepts both raw and gzipped FASTQ inputs.
    pub r1_src: PathBuf,

    /// Read 2 source. Accepts both raw and gzipped FASTQ inputs.
    pub r2_src: Option<PathBuf>,
}

#[derive(Parser)]
#[command(group(
    ArgGroup::new("quantity")
        .required(true)
        .args(["probability", "record_count", "record_count_per_tile"])
))]
pub struct SubsampleArgs {
    /// The probability a record is kept, as a percentage (0.0, 1.0). Cannot be used with
    /// `record-count` or `record-count-per-tile`. When combined with `--bin-by-tile`, the
    /// per-tile count is computed as floor(probability * total_records / num_bins).
    #[arg(short, long)]
    pub probability: Option<f64>,

    /// The exact number of records to keep. Cannot be used with `probability` or
    /// `record-count-per-tile`. When combined with `--bin-by-tile`, the per-tile count is
    /// computed as record_count / num_bins.
    #[arg(short = 'n', long)]
    pub record_count: Option<u64>,

    /// The exact number of records to keep per tile. Reads are binned by their lane and tile
    /// extracted from the Illumina read header. Bins with fewer than this many records are
    /// discarded, and exactly this many records are randomly sampled from each retained bin.
    /// Cannot be used with `probability` or `record-count`.
    #[arg(long)]
    pub record_count_per_tile: Option<u64>,

    /// Enable per-tile binning. Reads are binned by lane and tile from the Illumina read
    /// header. Can be combined with `--record-count` or `--probability` to automatically
    /// compute the per-tile count, or use `--record-count-per-tile` to set it explicitly.
    #[arg(long)]
    pub bin_by_tile: bool,

    /// Use faster skip-ahead sampling instead of the default exact method. Skip-ahead uses
    /// exponential byte jumps and produces approximately (not exactly) the requested number
    /// of records, but avoids reading the entire file.
    #[arg(long)]
    pub fast: bool,

    /// Keep tile bins in memory instead of writing to temporary files. Uses more memory but
    /// avoids temporary disk I/O. Only used with tile binning.
    #[arg(long)]
    pub in_memory: bool,

    /// Directory for temporary tile files. Defaults to the system temp directory. Only used
    /// with tile binning when --in-memory is not set.
    #[arg(long)]
    pub temp_dir: Option<PathBuf>,

    /// Number of threads for parallel tile sampling. Only used with tile binning.
    #[arg(long, default_value_t = 1)]
    pub sampling_threads: usize,

    /// Number of threads for output compression. Only used when output is gzipped.
    #[arg(long, default_value_t = 1)]
    pub compression_threads: usize,

    /// Seed to use for the random number generator.
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// Read 1 destination. Output will be gzipped if ends in `.gz`.
    #[arg(long)]
    pub r1_dst: PathBuf,

    /// Read 2 destination. Output will be gzipped if ends in `.gz`.
    #[arg(long)]
    pub r2_dst: Option<PathBuf>,

    /// Read 1 source. Accepts both raw and gzipped FASTQ inputs.
    pub r1_src: PathBuf,

    /// Read 2 source. Accepts both raw and gzipped FASTQ inputs.
    pub r2_src: Option<PathBuf>,
}
