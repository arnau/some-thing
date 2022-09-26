use serde::{Deserialize, Serialize};

use crate::entities::{tag, thing};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub timestamp: usize,
    pub change: Change,
}

#[derive(Debug, PartialEq)]
pub struct EventId {
    pub timestamp: usize,
    pub operation: String,
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operation")]
pub enum Change {
    Insert(Data),
    Replace(Data),
    Delete(DataRef),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum Data {
    Thing {
        #[serde(rename = "id")]
        url: thing::Id,
        name: String,
        summary: Option<String>,
        category: tag::Id,
        tags: Vec<tag::Id>,
    },
    Tag {
        id: tag::Id,
        name: Option<String>,
        summary: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum DataRef {
    Thing { id: thing::Id },
    Tag { id: tag::Id },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn baseline() {
        let ev = Event {
            timestamp: 1,
            change: Change::Insert(Data::Tag {
                id: "foo".to_string(),
                name: None,
                summary: None,
            }),
        };

        let actual: Event = serde_json::from_str(&serde_json::to_string(&ev).unwrap()).unwrap();

        assert_eq!(&ev, &actual);
    }
}
