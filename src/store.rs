use rusqlite::{Connection, Row, ToSql, Transaction, NO_PARAMS};
use std::include_str;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

use crate::tag::{Tag, TagId, TagSet};
use crate::thing::Thing;
use crate::Result;

pub const DEFAULT_PATH: &str = ":memory:";
pub const SCHEMA: &str = include_str!("./sql/bootstrap.sql");

pub type Tx<'a> = Transaction<'a>;

/// A strategy to connect to the storage.
#[derive(Debug, Clone, PartialEq)]
pub enum Strategy {
    Memory,
    Disk(PathBuf),
}

impl FromStr for Strategy {
    type Err = StoreError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            DEFAULT_PATH => Ok(Strategy::Memory),
            s => {
                let path = Path::new(s);
                Ok(Strategy::Disk(path.into()))
            }
        }
    }
}

#[derive(Debug)]
pub struct Store {
    strategy: Strategy,
    conn: Connection,
}

impl Store {
    pub fn open(strategy: &Strategy) -> Result<Self> {
        let conn = match strategy {
            Strategy::Memory => Connection::open_in_memory()?,
            Strategy::Disk(path) => Connection::open(path)?,
        };

        // conn.pragma_update(None, "journal_mode", &"wal")?;

        let store = Self {
            strategy: strategy.clone(),
            conn,
        };

        store.batch(&SCHEMA)?;

        Ok(store)
    }

    pub fn transaction(&mut self) -> Result<Transaction> {
        Ok(self.conn.transaction()?)
    }

    pub fn batch(&self, query: &str) -> Result<()> {
        self.conn.execute_batch(query)?;

        Ok(())
    }

    /// A query mapped over the given function.
    pub fn query<T, P, F>(&mut self, query: &str, params: P, f: F) -> Result<Vec<T>>
    where
        P: IntoIterator,
        P::Item: ToSql,
        F: FnMut(&Row<'_>) -> std::result::Result<T, rusqlite::Error>,
    {
        let mut stmt = self.conn.prepare(query)?;

        let rows = stmt.query_map(params, f)?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }
}

#[derive(Debug)]
pub struct ThingStore;

impl ThingStore {
    pub fn write(store: &mut Store, thing: Thing, tags: &[TagId]) -> Result<()> {
        let tx = store.transaction()?;

        let record: [&dyn ToSql; 4] = [
            &thing.url(),
            &thing.name(),
            &thing.summary(),
            &thing.category_id(),
        ];

        ThingStore::insert(&tx, &record)?;

        for tag in tags {
            let values: [&dyn ToSql; 2] = [&thing.url(), &tag];

            TagStore::insert(&tx, &values)?;
        }

        tx.commit()?;

        Ok(())
    }

    pub fn insert(tx: &Transaction, values: &[&dyn ToSql; 4]) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO thing
                (url, name, summary, category_id)
            VALUES
                (?, ?, ?, ?)
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TagStore;

impl TagStore {
    pub fn insert(tx: &Transaction, values: &[&dyn ToSql; 2]) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO thing_tag
                (thing_id, tag_id)
            VALUES
                (?, ?)
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    pub fn get_all(store: &mut Store) -> Result<TagSet> {
        let rows = store.query(r#"SELECT * FROM tag ORDER BY id"#, NO_PARAMS, |row| {
            Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        Ok(TagSet::new(rows))
    }

    pub fn get_by_thing(store: &mut Store, thing_id: &str) -> Result<TagSet> {
        let rows = store.query(
            r#"
        SELECT
            tag.*
        FROM
            thing_tag
        JOIN
            tag ON tag.id = thing_tag.tag_id
        WHERE
            thing_id = ?
        "#,
            &[thing_id],
            |row| Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        Ok(TagSet::new(rows))
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(
        "The given strategy `{0}` is not an acceptable path nor the special `:memory:` token."
    )]
    StrategyError(String),
}
