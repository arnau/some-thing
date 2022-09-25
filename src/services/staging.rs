use serde::Serialize;

use crate::{
    context::Context,
    entities::change::{Change, Data},
    store::{ChangeStore, Connection, Repository, TagStore, ThingStore},
    tag::{self, TagError},
    thing::{self, ThingError},
    Result, SomeError,
};
use std::{io::Write, ops::Deref};

/// Attempts to insert a new change to the store.
pub fn add(ctx: &mut Context, data: Data) -> Result<()> {
    let tx = ctx.tx()?;

    assert_data_exists(&tx, &data)?;

    let change = Change::Insert(data);

    ChangeStore::add(&tx, &change)?;

    tx.commit()?;

    Ok(())
}

/// Attempts to consume and commit all events in the changelog.
pub fn commit(ctx: &mut Context) -> Result<()> {
    let mut thing_file = ctx.open_resource("thing")?;
    let mut tag_file = ctx.open_resource("tag")?;
    let mut thing_tags_file = ctx.open_resource("thing_tag")?;
    let tx = ctx.tx()?;
    let changes = ChangeStore::to_vec(&tx)?;

    for change in changes {
        match change.change {
            Change::Insert(data) => match data {
                Data::Tag { id, name, summary } => {
                    let tag = tag::Record::new(id, name, summary);
                    write_once(&tag, &mut tag_file)?;
                }
                Data::Thing {
                    url,
                    name,
                    summary,
                    category,
                    tags,
                } => {
                    let thing_tags: Vec<(String, String)> = tags
                        .into_iter()
                        .map(|tag_id| (url.clone(), tag_id))
                        .collect();
                    let thing = thing::Record::new(url, name, summary, category);
                    write_once(&thing, &mut thing_file)?;
                    write_many(&thing_tags, &mut thing_tags_file)?;
                }
            },
            Change::Replace(_) => unimplemented!(),
            Change::Delete(_) => unimplemented!(),
        }
    }

    ChangeStore::flush(&tx)?;

    tx.commit()?;

    Ok(())
}

fn write_once<W, R>(record: &R, wtr: &mut W) -> Result<()>
where
    W: Write,
    R: Serialize,
{
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(wtr);

    wtr.serialize(&record)?;
    wtr.flush()?;

    Ok(())
}

fn write_many<W, R>(records: &[R], wtr: &mut W) -> Result<()>
where
    W: Write,
    R: Serialize,
{
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(wtr);

    for record in records {
        wtr.serialize(&record)?;
    }

    wtr.flush()?;

    Ok(())
}

fn assert_data_exists<C>(conn: &C, data: &Data) -> Result<()>
where
    C: Deref<Target = Connection>,
{
    match &data {
        Data::Thing { url, .. } => {
            if assert_thing_exists(conn, url)? {
                return Err(SomeError::Thing(ThingError::Duplicate(url.to_string())));
            }
        }
        Data::Tag { id, .. } => {
            if assert_tag_exists(conn, id)? {
                return Err(SomeError::Tag(TagError::Duplicate(id.to_string())));
            }
        }
    };

    Ok(())
}

/// Validate the given thing does not exist in the repository.
fn assert_thing_exists<C>(conn: &C, id: &thing::Id) -> Result<bool>
where
    C: Deref<Target = Connection>,
{
    Ok(ThingStore::get(conn, id)?.is_some())
}

/// Validate the given tag does exist in the repository.
fn assert_tag_exists<C>(conn: &C, id: &tag::Id) -> Result<bool>
where
    C: Deref<Target = Connection>,
{
    Ok(TagStore::get(conn, id)?.is_some())
}
