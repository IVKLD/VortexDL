use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum SyncMode {
    Silent,
    Full,
    Archive,
}

impl fmt::Display for SyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => write!(f, "silent"),
            Self::Full => write!(f, "full"),
            Self::Archive => write!(f, "archive"),
        }
    }
}
