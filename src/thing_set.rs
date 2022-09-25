use std::fmt;
use std::io::prelude::*;
use std::iter::FromIterator;

use crate::thing::{self, ThingError};

/// An iterable set of things.
#[derive(Debug, Clone)]
pub struct ThingSet(Vec<thing::Record>);

impl ThingSet {
    pub fn new(raw: Vec<thing::Record>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[thing::Record] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<thing::Record> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&thing::Record> {
        self.0.first()
    }

    /// Loads a ThingSet from a Reader. Must be a valid CSV.
    pub fn from_reader<R: Read>(rdr: &mut R) -> Result<Self, ThingError> {
        let mut rdr = csv::Reader::from_reader(rdr);
        let mut set = Vec::new();

        for result in rdr.deserialize() {
            let record: thing::Record = result?;

            set.push(record);
        }

        set.sort();

        Ok(Self(set))
    }
}

impl IntoIterator for ThingSet {
    type Item = thing::Record;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<thing::Record> for ThingSet {
    fn from_iter<I: IntoIterator<Item = thing::Record>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        ThingSet::new(v)
    }
}

impl fmt::Display for ThingSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list: Vec<String> = self.0.iter().map(|tag| format!("{}", tag)).collect();

        if f.alternate() {
            write!(f, "{}", list.join("\n"))
        } else {
            write!(f, "{}", list.join(";"))
        }
    }
}
