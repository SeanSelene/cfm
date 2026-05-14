use serde::{Deserialize, Serialize};
use std::fmt;
/// 链接模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LinkMode {
    #[default]
    Soft,
    Hard,
    Cp,
}

impl fmt::Display for LinkMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            LinkMode::Soft => "soft",
            LinkMode::Hard => "hard",
            LinkMode::Cp => "cp",
        };
        write!(f, "{}", s)
    }
}
