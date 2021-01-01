use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

use crate::tag::TagId;

/// A relation between a [`Thing`] and a [`Tag`].
///
/// [`Thing`]: crate::thing::Thing
/// [`Tag`]: crate::tag::Tag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thingtag {
    thing_id: String,
    tag_id: TagId,
}

impl Thingtag {
    pub fn new(thing_id: String, tag_id: TagId) -> Self {
        Self { thing_id, tag_id }
    }

    pub fn thing_id(&self) -> &str {
        &self.thing_id
    }

    pub fn tag_id(&self) -> &TagId {
        &self.tag_id
    }
}

#[derive(Error, Debug)]
pub enum ThingtagError {
    #[error("Unknown thingtag error")]
    Unknown,
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
