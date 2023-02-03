use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};
use git_testament::{git_testament, render_testament};

use crate::{validators::LintMode, ValidationLevel};

git_testament!(TESTAMENT);

#[derive(Parser)]
#[command(propagate_version = true, version = render_testament!(TESTAMENT))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, hide = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Filters a FASTQ from an allowlist of names.
    Filter(FilterArgs),
    /// Generates a random FASTQ file pair.
    Generate(GenerateArgs),
    /// Validates a FASTQ file pair.
    Lint(LintArgs),
    /// Outputs a subset of records.
    Subsample(SubsampleArgs),
}

#[derive(Parser)]
pub struct FilterArgs {
    /// Allowlist of record names.
    #[arg(long)]
    pub names: Option<PathBuf>,

    /// Source FASTQ.
    pub src: PathBuf,
}

#[derive(Parser)]
pub struct GenerateArgs {
    /// Seed to use for the random number generator.
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// Number of records to generate.
    #[arg(short = 'n', long, default_value_t = 10000)]
    pub record_count: u64,

    /// Number of bases in the sequence.
    #[arg(long, default_value_t = 101)]
    pub read_length: usize,

    /// Read 1 destination. Output will be gzipped if ends in `.gz`.
    pub r1_dst: PathBuf,

    /// Read 2 destination. Output will be gzipped if ends in `.gz`.
    pub r2_dst: PathBuf,
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

    /// Read 1 source. Accepts both raw and gzipped FASTQ inputs.
    pub r1_src: PathBuf,

    /// Read 2 source. Accepts both raw and gzipped FASTQ inputs.
    pub r2_src: Option<PathBuf>,
}

#[derive(Parser)]
#[command(group(
    ArgGroup::new("quantity")
        .required(true)
        .args(["probability", "record_count"])
))]
pub struct SubsampleArgs {
    /// The probability a record is kept, as a percentage (0.0, 1.0). Cannot be used with
    /// `record-count`.
    #[arg(short, long)]
    pub probability: Option<f64>,

    /// The exact number of records to keep. Cannot be used with `probability`.
    #[arg(short = 'n', long)]
    pub record_count: Option<u64>,

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
