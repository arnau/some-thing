use clap::Clap;
use rusqlite::{Connection, ToSql, Transaction, NO_PARAMS};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::store::{Store, DEFAULT_PATH};
use crate::tag::{Tag, TagId};
use crate::thing::Thing;
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
        let mut store = Store::open(&self.store_path)?;
        let tx = store.transaction()?;
        let mut writer = io::stdout();

        process(&tx, &mut writer)?;

        tx.commit()?;

        Ok(Report::new(""))
    }
}

fn process<W: Write>(tx: &Transaction, writer: &mut W) -> Result<()> {
    writeln!(writer, "# Some FIXME\n")?;
    writeln!(writer, "Some FIXME description\n")?;

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
            writeln!(writer, "|-|-|-|")?;

            for thing in things {
                let link = format!("[{}]({})", &thing.name(), &thing.url());
                let tags = "";
                let summary = &thing.summary();
                writeln!(
                    writer,
                    "| {} | {} | {} |",
                    link,
                    summary.unwrap_or(&"".to_string()),
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
            tag.summary,
            tag.icon
        FROM
            thing
        JOIN
            tag ON tag.id = thing.category_id
        "#,
    )?;

    let rows = stmt.query_map(NO_PARAMS, |row| {
        Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
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
        let tags = get_thing_tags(tx, &url)?;
        let thing = Thing::new(url, row.get(1)?, tags, row.get(2)?);

        list.push(thing);
    }

    Ok(list)
}

fn get_thing_tags(tx: &Transaction, thing_id: &str) -> Result<Vec<TagId>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT
            tag_id
        FROM
            thing_tag
        WHERE
            thing_id = ?
        "#,
    )?;
    let rows = stmt.query_map(&[thing_id], |row| {
        let tag: TagId = row.get(0)?;

        Ok(tag)
    })?;

    let mut list = Vec::new();

    for result in rows {
        list.push(result?);
    }

    Ok(list)
}
