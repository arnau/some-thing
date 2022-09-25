use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use thiserror::Error;

pub type Id = String;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Record {
    id: Id,
    name: Option<String>,
    summary: Option<String>,
}

impl Record {
    pub fn new(id: Id, name: Option<String>, summary: Option<String>) -> Self {
        Self { id, name, summary }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Error, Debug)]
pub enum TagError {
    #[error("Unknown tag error")]
    Unknown,
    #[error("A tag '{0}' already exists.")]
    Duplicate(String),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
