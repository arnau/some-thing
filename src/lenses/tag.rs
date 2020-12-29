use skim::prelude::*;

use crate::store::{Store, TagStore};
use crate::tag::{Tag, TagSet};
use crate::Result;

/// The full set of tags in the store.
pub fn full_set(store: &mut Store) -> Result<TagSet> {
    Ok(TagStore::get_all(store)?)
}

impl SkimItem for Tag {
    fn display(&self) -> Cow<AnsiString> {
        Cow::Owned(self.id().into())
    }

    fn text(&self) -> Cow<str> {
        Cow::Borrowed(self.id())
    }

    fn preview(&self) -> ItemPreview {
        ItemPreview::Text(format!(
            "{}: {}",
            self.name().unwrap_or(&self.id().to_string()),
            self.summary().unwrap_or(&"".to_string())
        ))
    }
}

impl TagSet {
    pub fn as_skim_buffer(&self) -> SkimItemReceiver {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        for tag in self.to_vec() {
            let _ = tx_item.send(Arc::new(tag));
        }

        drop(tx_item);

        rx_item
    }
}

impl From<TagSet> for SkimItemReceiver {
    fn from(input: TagSet) -> SkimItemReceiver {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        for tag in input {
            let _ = tx_item.send(Arc::new(tag));
        }

        drop(tx_item);

        rx_item
    }
}
