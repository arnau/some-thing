use serde::{Deserialize, Serialize};
use std::{fmt, io};
use thiserror::Error;

use crate::entities::tag;

pub type Id = String;

/// Represents a self contained thing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Thing {
    #[serde(rename = "id")]
    pub url: Id,
    pub name: String,
    pub summary: Option<String>,
    pub category: tag::Id,
    pub tags: Vec<tag::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Record {
    url: Id,
    name: String,
    #[serde(with = "empty_string")]
    summary: Option<String>,
    category_id: tag::Id,
}

impl Record {
    pub fn new(url: Id, name: String, summary: Option<String>, category_id: tag::Id) -> Self {
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

    pub fn category_id(&self) -> &tag::Id {
        &self.category_id
    }
}

impl fmt::Display for Record {
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
    #[error("A thing exists with the URL '{0}'")]
    Duplicate(String),
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
