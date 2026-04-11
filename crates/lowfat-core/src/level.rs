use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Intensity level for filtering.
/// lite = gentle trim, full = default, ultra = max compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Lite,
    #[default]
    Full,
    Ultra,
}

impl Level {
    /// Scale a base head-limit by level.
    /// lite = 2x, full = base, ultra = max(base/2, 5)
    pub fn head_limit(&self, base: usize) -> usize {
        match self {
            Level::Lite => base * 2,
            Level::Full => base,
            Level::Ultra => (base / 2).max(5),
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Lite => write!(f, "lite"),
            Level::Full => write!(f, "full"),
            Level::Ultra => write!(f, "ultra"),
        }
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lite" => Ok(Level::Lite),
            "full" => Ok(Level::Full),
            "ultra" => Ok(Level::Ultra),
            _ => Err(format!("unknown level: {s} (expected: lite, full, ultra)")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_limit_lite() {
        assert_eq!(Level::Lite.head_limit(40), 80);
    }

    #[test]
    fn head_limit_full() {
        assert_eq!(Level::Full.head_limit(40), 40);
    }

    #[test]
    fn head_limit_ultra() {
        assert_eq!(Level::Ultra.head_limit(40), 20);
    }

    #[test]
    fn head_limit_ultra_minimum() {
        // base=8 -> 8/2=4, but min is 5
        assert_eq!(Level::Ultra.head_limit(8), 5);
    }

    #[test]
    fn from_str() {
        assert_eq!("lite".parse::<Level>().unwrap(), Level::Lite);
        assert_eq!("FULL".parse::<Level>().unwrap(), Level::Full);
        assert_eq!("Ultra".parse::<Level>().unwrap(), Level::Ultra);
        assert!("invalid".parse::<Level>().is_err());
    }

    #[test]
    fn display() {
        assert_eq!(Level::Lite.to_string(), "lite");
        assert_eq!(Level::Full.to_string(), "full");
        assert_eq!(Level::Ultra.to_string(), "ultra");
    }
}
