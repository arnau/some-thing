use clap::Parser;
use std::path::PathBuf;

use super::Prompter;
use crate::context::Context;
use crate::entities::change::Data;
use crate::lenses;
use crate::services::staging;
use crate::store::{Repository, TagStore, ThingStore};
use crate::tag_set::TagSet;
use crate::{Report, Result};

/// Adds a new item to the collection.
#[derive(Debug, Parser)]
pub struct Cmd {
    /// The location where to find the Some package to be used.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut prompter = Prompter::new()?;
        let mut context = Context::new(&self.path)?;
        let store = context.store();

        // Main info
        let url = prompter.demand("url")?;

        // TODO: Consider offering the option to amend it.
        if ThingStore::get(&store.conn, &url)?.is_some() {
            return Ok(Report::new("This thing already exists."));
        }

        // TODO: Move to a new service 'fetcher'.
        lenses::thing::fetch_thing(&url)?;

        let name = prompter.demand("name")?;
        let summary = prompter.ask_once("summary")?;

        // Category
        let category_set = TagSet::from_iter(TagStore::list(&store.conn)?);
        let default_category_id = "miscellaneous";
        let category_id = ask_category(&mut prompter, default_category_id, category_set);

        // Tags
        let tag_set = TagSet::from_iter(TagStore::list_without(
            &store.conn,
            &[default_category_id.into()],
        )?);
        let tags = ask_tags(&mut prompter, tag_set);

        prompter.flush()?;

        // Build the thing
        let data = Data::Thing {
            url,
            name,
            summary,
            category: category_id,
            tags,
        };

        staging::add(&mut context, data)?;

        staging::commit(&mut context)?;

        Ok(Report::new("Success"))
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
