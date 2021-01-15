use std::fmt;
use std::io::prelude::*;
use std::iter::FromIterator;

use crate::thing::{Thing, ThingError};

/// An iterable set of things.
#[derive(Debug, Clone)]
pub struct ThingSet(Vec<Thing>);

impl ThingSet {
    pub fn new(raw: Vec<Thing>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[Thing] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Thing> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Thing> {
        self.0.first()
    }

    /// Loads a ThingSet from a Reader. Must be a valid CSV.
    pub fn from_reader<R: Read>(rdr: &mut R) -> Result<Self, ThingError> {
        let mut rdr = csv::Reader::from_reader(rdr);
        let mut set = Vec::new();

        for result in rdr.deserialize() {
            let record: Thing = result?;

            set.push(record);
        }

        set.sort();

        Ok(Self(set))
    }
}

impl IntoIterator for ThingSet {
    type Item = Thing;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Thing> for ThingSet {
    fn from_iter<I: IntoIterator<Item = Thing>>(iter: I) -> Self {
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
