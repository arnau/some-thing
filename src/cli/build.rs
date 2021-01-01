use clap::Clap;
use rusqlite::{Transaction, NO_PARAMS};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::context::Context;
use crate::store::{Store, Strategy, DEFAULT_PATH};
use crate::tag::{Tag, TagId};
use crate::thing::Thing;
use crate::{Report, Result};

/// Build the Markdown version of the collection.
#[derive(Debug, Clap)]
pub struct Cmd {
    /// The path to the cache.
    #[clap(long, value_name = "path", default_value = DEFAULT_PATH)]
    cache: Strategy,
    /// The location where to find the Some package to be destroyed.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let context = Context::new(&self.path)?;

        let mut store = Store::open(&self.cache)?;
        let tx = store.transaction()?;
        let mut writer = io::stdout();

        process(&tx, &context, &mut writer)?;

        tx.commit()?;

        Ok(Report::new(""))
    }
}

fn process<W: Write>(tx: &Transaction, context: &Context, writer: &mut W) -> Result<()> {
    let package = context.package();
    writeln!(writer, "# {}\n", package.name())?;
    writeln!(writer, "{}\n", package.description())?;

    let categories = get_categories(tx)?;

    for category in categories {
        let category_id = category.id().to_string();
        let name = category.name().unwrap_or(&category_id);
        let things = get_category_things(tx, &category_id)?;

        if !&things.is_empty() {
            writeln!(writer, "## {}\n", name)?;
            if let Some(summary) = category.summary() {
                writeln!(writer, "{}\n", summary)?;
            }

            writeln!(writer, "| name | summary | tags |")?;
            writeln!(writer, "| - | - | - |")?;

            for thing in things {
                let link = format!("[{}]({})", &thing.name(), &thing.url());
                let tags = "";
                let summary = &thing.summary();
                writeln!(
                    writer,
                    "| {} | {} | {} |",
                    link,
                    summary.as_ref().unwrap_or(&"".to_string()),
                    tags
                )?;
            }

            writeln!(writer, "")?;
        }
    }

    Ok(())
}

fn get_categories(tx: &Transaction) -> Result<Vec<Tag>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT DISTINCT
            tag.id,
            tag.name,
            tag.summary
        FROM
            thing
        JOIN
            tag ON tag.id = thing.category_id
        "#,
    )?;

    let rows = stmt.query_map(NO_PARAMS, |row| {
        Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    let mut list = Vec::new();

    for result in rows {
        list.push(result?);
    }

    Ok(list)
}

fn get_category_things(tx: &Transaction, category_id: &TagId) -> Result<Vec<Thing>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT DISTINCT
            url,
            name,
            summary
        FROM
           thing
        WHERE
            category_id = ?
        "#,
    )?;

    let mut rows = stmt.query(&[category_id])?;
    let mut list = Vec::new();

    while let Some(row) = rows.next()? {
        let url: String = row.get(0)?;
        // let tags = get_thing_tags(tx, &url)?;
        let thing = Thing::new(url, row.get(1)?, row.get(2)?, category_id.clone());

        list.push(thing);
    }

    Ok(list)
}
