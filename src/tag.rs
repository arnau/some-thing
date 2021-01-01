use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::prelude::*;
use std::iter::FromIterator;

use crate::Result;

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

#[derive(Debug, Clone)]
pub struct TagSet(Vec<Tag>);

impl TagSet {
    pub fn new(raw: Vec<Tag>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[Tag] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Tag> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Tag> {
        self.0.first()
    }

    /// Loads a TagSet from a Reader. Must be a valid CSV.
    pub fn from_reader<R: Read>(rdr: &mut R) -> Result<Self> {
        let mut rdr = csv::Reader::from_reader(rdr);
        let mut set = Vec::new();

        for result in rdr.deserialize() {
            let record: Tag = result?;

            set.push(record);
        }

        Ok(Self(set))
    }
}

impl IntoIterator for TagSet {
    type Item = Tag;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Tag> for TagSet {
    fn from_iter<I: IntoIterator<Item = Tag>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        TagSet::new(v)
    }
}

impl fmt::Display for TagSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list: Vec<String> = self.0.iter().map(|tag| format!("{}", tag)).collect();

        if f.alternate() {
            write!(f, "{}", list.join("\n"))
        } else {
            write!(f, "{}", list.join(";"))
        }
    }
}
