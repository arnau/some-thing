use std::io::prelude::*;
use std::iter::FromIterator;

use crate::thingtag::{Thingtag, ThingtagError};

/// An iterable set of things.
#[derive(Debug, Clone)]
pub struct ThingtagSet(Vec<Thingtag>);

impl ThingtagSet {
    pub fn new(raw: Vec<Thingtag>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[Thingtag] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Thingtag> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Thingtag> {
        self.0.first()
    }

    /// Loads a ThingSet from a Reader. Must be a valid CSV.
    pub fn from_reader<R: Read>(rdr: &mut R) -> Result<Self, ThingtagError> {
        let mut rdr = csv::Reader::from_reader(rdr);
        let mut set = Vec::new();

        for result in rdr.deserialize() {
            let record: Thingtag = result?;

            set.push(record);
        }

        Ok(Self(set))
    }
}

impl IntoIterator for ThingtagSet {
    type Item = Thingtag;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Thingtag> for ThingtagSet {
    fn from_iter<I: IntoIterator<Item = Thingtag>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        ThingtagSet::new(v)
    }
}
