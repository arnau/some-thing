use clap::Parser;
use std::io::prelude::*;
use std::path::PathBuf;

use super::Prompter;
use crate::context::Context;
use crate::lenses;
use crate::store::{Store, Strategy};
use crate::tag_set::TagSet;
use crate::thing::Thing;
use crate::thing_set::ThingSet;
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
        let context = Context::new(&self.path)?;
        let mut thing_file = context.open_resource("thing")?;
        let mut tag_file = context.open_resource("tag")?;
        let mut thingtag_file = context.open_resource("thing_tag")?;
        let thingset = ThingSet::from_reader(&mut thing_file)?;

        let mut store = Store::open(self.path.to_path_buf(), &Strategy::Memory)?;

        let r = store.query(
            "select * from staging.sqlite_schema",
            [],
            |row| {
                let url: String = row.get(0)?;
                let name: String = row.get(1)?;
                let summary: Option<String> = row.get(2)?;

                Ok((url, name, summary))
            },
        )?;

        dbg!(r);

        // Main info
        let url = prompter.demand("url")?;

        if thingset.into_iter().find(|t| t.url() == url).is_some() {
            return Ok(Report::new("This thing already exists."));
        }

        lenses::thing::fetch_thing(&url)?;

        let name = prompter.demand("name")?;
        let summary = prompter.ask_once("summary")?;

        // Category
        let category_items = TagSet::from_reader(&mut tag_file)?;
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
        let thing = Thing::new(url, name, summary, category_id);

        // Write
        write_thing(&thing, &mut thing_file)?;
        write_thingtags(&thing, &tags, &mut thingtag_file)?;

        let report = Report::new("Success");
        Ok(report)
    }
}

fn write_thing<W: Write>(thing: &Thing, wtr: &mut W) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(wtr);

    wtr.serialize(&thing)?;
    wtr.flush()?;

    Ok(())
}

fn write_thingtags<W: Write>(thing: &Thing, tags: &[String], wtr: &mut W) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(wtr);

    for tag in tags {
        let record = vec![thing.url(), &tag];
        wtr.write_record(&record)?;
    }

    wtr.flush()?;

    Ok(())
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
