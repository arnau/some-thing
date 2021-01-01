use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use thiserror::Error;

pub type TagId = String;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: TagId,
    name: Option<String>,
    summary: Option<String>,
}

impl Tag {
    pub fn new(id: TagId, name: Option<String>, summary: Option<String>) -> Self {
        Self { id, name, summary }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Error, Debug)]
pub enum TagError {
    #[error("Unknown tag error")]
    Unknown,
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
