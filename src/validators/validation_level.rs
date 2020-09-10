use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationLevel {
    Low,
    Medium,
    High,
}

impl FromStr for ValidationLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(ValidationLevel::Low),
            "medium" => Ok(ValidationLevel::Medium),
            "high" => Ok(ValidationLevel::High),
            _ => Err(()),
        }
    }
}
