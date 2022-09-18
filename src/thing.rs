use serde::{Deserialize, Serialize};
use std::{fmt, io};
use thiserror::Error;

use crate::tag::TagId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Thing {
    url: String,
    name: String,
    #[serde(with = "empty_string")]
    summary: Option<String>,
    category_id: TagId,
}

impl Thing {
    pub fn new(url: String, name: String, summary: Option<String>, category_id: TagId) -> Self {
        Self {
            url,
            name,
            summary,
            category_id,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    pub fn category_id(&self) -> &TagId {
        &self.category_id
    }
}

impl fmt::Display for Thing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// impl Markdown for Thing {
//     fn fmt_md(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
//         write!(f, "")

//     }
// }

mod empty_string {
    use serde::Deserialize;

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *value {
            Some(ref v) => serializer.serialize_some(v),
            None => serializer.serialize_some(""),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let opt = if value.trim().is_empty() {
            None
        } else {
            Some(value.trim().to_string())
        };

        Ok(opt)
    }
}

#[derive(Error, Debug)]
pub enum ThingError {
    #[error("'url' is a required field")]
    MissingUrl,
    #[error("'name' is a required field")]
    MissingName,
    #[error("'category_id' is a required field")]
    MissingCategory,
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
