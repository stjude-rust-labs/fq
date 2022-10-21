#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, clap::ValueEnum)]
pub enum ValidationLevel {
    Low,
    Medium,
    High,
}
