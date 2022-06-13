use std::{error, fmt, str::FromStr};

use clap::PossibleValue;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid input: '{}'", self.0)
    }
}

impl FromStr for ValidationLevel {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(ParseError(s.into())),
        }
    }
}

impl clap::ValueEnum for ValidationLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Low, Self::Medium, Self::High]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Self::Low => Some(PossibleValue::new("low")),
            Self::Medium => Some(PossibleValue::new("medium")),
            Self::High => Some(PossibleValue::new("high")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!("low".parse(), Ok(ValidationLevel::Low));
        assert_eq!("medium".parse(), Ok(ValidationLevel::Medium));
        assert_eq!("high".parse(), Ok(ValidationLevel::High));

        assert_eq!(
            "".parse::<ValidationLevel>(),
            Err(ParseError(String::new()))
        );
        assert_eq!(
            "Low".parse::<ValidationLevel>(),
            Err(ParseError(String::from("Low")))
        );
        assert_eq!(
            "LOW".parse::<ValidationLevel>(),
            Err(ParseError(String::from("LOW")))
        );
        assert_eq!(
            "fqlib".parse::<ValidationLevel>(),
            Err(ParseError(String::from("fqlib")))
        );
    }
}
