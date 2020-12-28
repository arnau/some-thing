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
        let mut thing = NewThingBuilder::new();

        // Main info
        let url = prompter.demand("url")?;
        lenses::thing::fetch_thing(&url)?;
        let name = prompter.demand("name")?;
        let summary = prompter.ask_once("summary")?;

        // Category
        let category_items = lenses::tag::full_set(&mut store)?;
        let default_category_id = "miscellaneous";
        let category_id = ask_category(&mut prompter, default_category_id, category_items.clone());

        // Tags
        let tag_items = category_items
            .into_iter()
            .filter(|tag| tag.id() != category_id)
            .collect::<TagSet>();
        let tags = ask_tags(&mut prompter, tag_items);

        prompter.flush()?;

        // Build the thing
        thing.with_url(url);
        thing.with_name(name);
        if let Some(summary) = summary {
            thing.with_summary(summary);
        };
        thing.with_category_id(category_id);
        thing.with_tags(&tags);

        let report = lenses::thing::add(&mut store, thing.build()?)?;

        Ok(report)
    }
}

/// Ask for a category or fallback to the default category.
fn ask_category(prompter: &mut Prompter, default_id: &str, items: TagSet) -> String {
    if items.len() == 1 {
        items.first().expect("always an item present").to_string()
    } else {
        prompter
            .read_choice(items.clone(), "category")
            .expect("always to read a choice")
            .unwrap_or(default_id.into())
    }
}

/// Ask to choose zero or more tags.
fn ask_tags(prompter: &mut Prompter, items: TagSet) -> Vec<String> {
    if items.len() == 0 {
        Vec::new()
    } else {
        prompter
            .read_choices(items, "tags")
            .expect("always to read a tag choice")
    }
}
