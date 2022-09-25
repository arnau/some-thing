//! Store module
//!
//! The store physical data model is split in two rings: the source and the staging.
//!
//! The source contains the set of tables derived from the CSV files found in the package.
//!
//! The staging contains the set of tables with the new data, not yet persisted in the source.
//!
//! The rings are implemented as distinct SQLite databases such that the main database contains just convenience views to query both rings.

pub use rusqlite::{params, Connection};
use rusqlite::{Row, Transaction};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

use crate::Result;

mod tag;
mod thing;
mod thing_tag;
mod change;
pub use tag::TagStore;
pub use thing::ThingStore;
pub use thing_tag::ThingtagStore;
pub use change::ChangeStore;

pub const DEFAULT_PATH: &str = ":memory:";

#[macro_export]
macro_rules! table {
    ("tag", $ring:expr) => {
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS '{ring}'.tag (
                id      text NOT NULL,
                name    text,
                summary text,

                PRIMARY KEY (id)
            );"#,
            ring = $ring
        )
    };

    ("thing", $ring:expr) => {
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS '{ring}'.thing (
                url         text NOT NULL,
                name        text NOT NULL,
                summary     text,
                category_id text NOT NULL,

                PRIMARY KEY (url)
            );"#,
            ring = $ring
        )
    };

    ("thing_tag", $ring:expr) => {
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS '{ring}'.thing_tag (
                thing_id text NOT NULL,
                tag_id   text NOT NULL,

                PRIMARY KEY (thing_id, tag_id)
            );
            "#,
            ring = $ring
        )
    };
}

#[macro_export]
macro_rules! virtual_table {
    ($name:expr, $filename:expr, $schema:expr) => {
        format!(
            r#"
            CREATE VIRTUAL TABLE source.'{name}'
                USING csv(filename='{filename}', header='yes', schema='{schema}');
            "#,
            name = $name,
            filename = $filename.canonicalize()?.to_str().unwrap(),
            schema = $schema,
        )
    };
}

// TODO: this approach does not handle deletes.
pub const OVERLAY_SCHEMA: &str = r#"
CREATE TEMPORARY VIEW tag AS
    SELECT
        json_extract(data, '$.id') AS id,
        json_extract(data, '$.name') AS name,
        json_extract(data, '$.summary') AS summary
    FROM staging.changelog
    WHERE
        kind = 'tag'
    AND
        operation = 'insert'

    UNION ALL
    SELECT
        id,
        name,
        iif(summary = '', NULL, summary)
    FROM source.tag;

CREATE TEMPORARY VIEW thing AS
    SELECT
        json_extract(data, '$.id') AS url,
        json_extract(data, '$.name') AS name,
        json_extract(data, '$.summary') AS summary,
        json_extract(data, '$.category') AS category_id
    FROM staging.changelog
    WHERE
        kind = 'thing'
    AND
        operation = 'insert'
    UNION ALL
    SELECT
        url,
        name,
        iif(summary = '', NULL, summary),
        category_id
    FROM source.thing;

CREATE TEMPORARY VIEW thing_tag AS
    SELECT * FROM staging.thing_tag
    UNION ALL
    SELECT * FROM source.thing_tag;
"#;

pub type Tx<'a> = Transaction<'a>;

/// High level interface for stores.
pub trait Repository<'a> {
    type Entity;
    type EntityId;
    type Conn: Deref<Target = Connection>;

    // Read
    fn get(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<Option<Self::Entity>>;
    fn to_vec(conn: Self::Conn) -> Result<Vec<Self::Entity>>;
    fn len(conn: Self::Conn) -> Result<usize>;

    fn is_empty(conn: Self::Conn) -> Result<bool> {
        Self::len(conn).map(|x| x == 0)
    }
    fn contains(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<bool> {
        Self::get(conn, entity_id).map(|e| e.is_some())
    }

    // Write
    fn add(conn: Self::Conn, entity: &Self::Entity) -> Result<()>;
    fn remove(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<()>;
    fn replace(conn: Self::Conn, entity: &Self::Entity) -> Result<()>;
}

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
    pub strategy: Strategy,
    pub path: PathBuf,
    /// DB connection.
    pub conn: Connection,
}

impl Store {
    pub fn open(path: PathBuf, strategy: &Strategy) -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        create_source_db(&conn, &path)?;
        create_staging_db(&conn, &path, strategy)?;

        conn.execute_batch(&OVERLAY_SCHEMA)?;

        let store = Self {
            strategy: strategy.clone(),
            path,
            conn,
        };

        Ok(store)
    }

    pub fn transaction(&mut self) -> Result<Transaction> {
        Ok(self.conn.transaction()?)
    }

    /// A query mapped over the given function.
    pub fn query<T, P, F>(&mut self, query: &str, params: P, f: F) -> Result<Vec<T>>
    where
        P: rusqlite::Params,
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

fn create_source_db(conn: &Connection, path: &Path) -> Result<()> {
    rusqlite::vtab::csvtab::load_module(&conn)?;

    let tag_schema = virtual_table!("tag", path.join("data/tag.csv"), table!("tag", "source"));
    let thing_schema = virtual_table!(
        "thing",
        path.join("data/thing.csv"),
        table!("thing", "source")
    );
    let thing_tag_schema = virtual_table!(
        "thing_tag",
        path.join("data/thing_tag.csv"),
        table!("thing_tag", "source")
    );

    let schema = format!(
        r#"
        ATTACH DATABASE ':memory:' AS source;

        CREATE TABLE IF NOT EXISTS source.package (
            body text NOT NULL,
            hash text NOT NULL,
            id   text GENERATED ALWAYS AS (json_extract(body, '$.id')) VIRTUAL NOT NULL UNIQUE
        );

        {} {} {}
        "#,
        tag_schema, thing_schema, thing_tag_schema
    );

    conn.execute_batch(&schema)?;

    Ok(())
}

// TODO: If changelog has the data as a json blob, I can get rid of the staging ring
// alltogether.
fn create_staging_db(conn: &Connection, path: &Path, strategy: &Strategy) -> Result<()> {
    let path = match strategy {
        Strategy::Memory => ":memory:".to_string(),
        Strategy::Disk(_) => path.join(".some.db").to_str().unwrap().to_string(),
    };

    let schema = format!(
        r#"
        ATTACH DATABASE '{}' AS staging;

        CREATE TABLE staging.changelog (
            timestamp datetime DEFAULT (datetime('now')),
            data      text NOT NULL,
            operation text GENERATED ALWAYS AS (lower(json_extract(data, '$.operation'))) NOT NULL,
            kind      text GENERATED ALWAYS AS (lower(json_extract(data, '$.kind'))) NOT NULL,
            id        text GENERATED ALWAYS AS (json_extract(data, '$.id')) NOT NULL
        );

        {} {} {}
        "#,
        path,
        table!("tag", "staging"),
        table!("thing", "staging"),
        table!("thing_tag", "staging"),
    );

    conn.execute_batch(&schema)?;

    Ok(())
}

// TODO: REVIEW
// fn get_categories(tx: &Transaction) -> Result<Vec<Tag>> {
//     let mut stmt = tx.prepare(
//         r#"
//         SELECT DISTINCT
//             tag.id,
//             tag.name,
//             tag.summary
//         FROM
//             thing
//         JOIN
//             tag ON tag.id = thing.category_id
//         "#,
//     )?;

//     let rows = stmt.query_map(NO_PARAMS, |row| {
//         Ok(Tag::new(row.get(0)?, row.get(1)?, row.get(2)?))
//     })?;

//     let mut list = Vec::new();

//     for result in rows {
//         list.push(result?);
//     }

//     Ok(list)
// }

// fn get_category_things(tx: &Transaction, category_id: &TagId) -> Result<Vec<Thing>> {
//     let mut stmt = tx.prepare(
//         r#"
//         SELECT DISTINCT
//             url,
//             name,
//             summary
//         FROM
//            thing
//         WHERE
//             category_id = ?
//         "#,
//     )?;

//     let mut rows = stmt.query(&[category_id])?;
//     let mut list = Vec::new();

//     while let Some(row) = rows.next()? {
//         let url: String = row.get(0)?;
//         // let tags = get_thing_tags(tx, &url)?;
//         let thing = Thing::new(url, row.get(1)?, row.get(2)?, category_id.clone());

//         list.push(thing);
//     }

//     Ok(list)
// }

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(
        "The given strategy `{0}` is not an acceptable path nor the special `:memory:` token."
    )]
    StrategyError(String),
    #[error("The given query expected a non-empty result:\n\n{0}")]
    EmptyError(String),
}
