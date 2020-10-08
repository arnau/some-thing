use rusqlite::{Connection, ToSql, Transaction, NO_PARAMS};
use std::include_str;
use std::path::{Path, PathBuf};

use crate::tag::{Tag, TagSet};
use crate::thing::NewThing;
use crate::Result;

pub static DEFAULT_PATH: &str = "./some.db";
pub static SCHEMA: &str = include_str!("./sql/bootstrap.sql");

pub type Tx<'a> = Transaction<'a>;

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    conn: Connection,
}

impl Store {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(&path)?;
        // conn.pragma_update(None, "journal_mode", &"wal")?;

        let store = Self {
            path: path.as_ref().into(),
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
}

#[derive(Debug)]
pub struct ThingStore;

impl ThingStore {
    pub fn write(tx: &Transaction, entity: NewThing) -> Result<()> {
        let thing: [&dyn ToSql; 4] = [
            &entity.url(),
            &entity.name(),
            &entity.summary(),
            &entity.category_id(),
        ];

        ThingStore::insert(tx, &thing)?;

        for tag in entity.tags() {
            let values: [&dyn ToSql; 2] = [&entity.url(), &tag];

            ThingStore::insert_tag(tx, &values)?;
        }

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

    pub fn insert_tag(tx: &Transaction, values: &[&dyn ToSql; 2]) -> Result<()> {
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
}

#[derive(Debug)]
pub struct TagStore;

impl TagStore {
    pub fn re(tx: &Transaction, values: &[&dyn ToSql; 2]) -> Result<()> {
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

    pub fn get_all(tx: &Transaction) -> Result<TagSet> {
        let mut stmt = tx.prepare(r#"SELECT * FROM tag ORDER BY id"#)?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
            Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut tagset = Vec::new();

        for result in rows {
            tagset.push(result?);
        }

        Ok(TagSet::new(tagset))
    }
}
