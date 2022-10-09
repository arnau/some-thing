use std::str::FromStr;
use std::fmt;
use anyhow::anyhow;

/// Composes the possible Shell configuration options.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub output_mode: OutputMode,
}




#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OutputMode {
    Tabbed,
    Table,
    Jsonline,
}

impl Default for OutputMode {
    fn default() -> Self {
        OutputMode::Tabbed
    }
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tabbed => write!(f, "tabbed"),
            Self::Table => write!(f, "table"),
            Self::Jsonline => write!(f, "jsonline"),
        }
    }
}

impl FromStr for OutputMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mode = match s {
            "tabbed" => OutputMode::Tabbed,
            "table" => OutputMode::Table,
            "jsonline" => OutputMode::Jsonline,
            value => {
                return Err(anyhow!("{} is not a valid mode", value));
            }
        };

        Ok(mode)
    }
}
