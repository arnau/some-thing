use clap::Clap;
use std::path::PathBuf;

use super::Prompter;
use crate::lenses;
use crate::store::{Store, DEFAULT_PATH};
use crate::tag::TagSet;
use crate::thing::NewThingBuilder;
use crate::{Report, Result};

/// Add a new item to the collection.
#[derive(Debug, Clap)]
pub struct Cmd {
    /// Store path
    #[clap(long, value_name = "path", default_value = DEFAULT_PATH)]
    store_path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut prompter = Prompter::new()?;

        let mut store = Store::open(&self.store_path)?;
        let tx = store.transaction()?;
        let mut thing = NewThingBuilder::new();

        let url = prompter.demand("url")?;
        lenses::thing::fetch_thing(&url)?;
        let name = prompter.demand("name")?;
        let summary = prompter.ask_once("summary")?;

        let category_items = lenses::tag::full_set(&tx)?;

        let category_id = if category_items.len() == 1 {
            category_items
                .first()
                .expect("always an item present")
                .to_string()
        } else {
            prompter
                .read_choice(category_items.clone(), "category")?
                .unwrap_or("miscellaneous".into())
        };

        let tag_items = category_items
            .into_iter()
            .filter(|tag| tag.id() != &category_id)
            .collect::<TagSet>();

        let tags = if tag_items.len() == 0 {
            Vec::new()
        } else {
            prompter.read_choices(tag_items, "tags")?
        };

        thing.with_url(url);
        thing.with_name(name);
        if let Some(summary) = summary {
            thing.with_summary(summary);
        };
        thing.with_category_id(category_id);
        thing.with_tags(&tags);

        dbg!(&thing);

        // let report = lenses::thing::add(&tx)?;

        prompter.flush()?;
        tx.commit()?;

        Ok(Report::new("Success"))
    }
}
