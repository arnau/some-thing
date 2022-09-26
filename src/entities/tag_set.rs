use std::fmt;
use std::io::prelude::*;
use std::iter::FromIterator;

use crate::entities::tag::{self, TagError};

#[derive(Debug, Clone)]
pub struct TagSet(Vec<tag::Record>);

impl TagSet {
    pub fn new(raw: Vec<tag::Record>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[tag::Record] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<tag::Record> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&tag::Record> {
        self.0.first()
    }

    /// Loads a TagSet from a Reader. Must be a valid CSV.
    pub fn from_reader<R: Read>(rdr: &mut R) -> Result<Self, TagError> {
        let mut rdr = csv::Reader::from_reader(rdr);
        let mut set = Vec::new();

        for result in rdr.deserialize() {
            let record: tag::Record = result?;

            set.push(record);
        }

        Ok(Self(set))
    }
}

impl IntoIterator for TagSet {
    type Item = tag::Record;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<tag::Record> for TagSet {
    fn from_iter<I: IntoIterator<Item = tag::Record>>(iter: I) -> Self {
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
