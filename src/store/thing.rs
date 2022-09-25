use std::ops::Deref;

use super::{params, Connection, Repository};
use crate::tag;
use crate::thing;
use crate::Result;

#[derive(Debug)]
pub struct ThingStore;

impl<'a> Repository<'a> for ThingStore {
    type Entity = thing::Record;
    type EntityId = thing::Id;
    type Conn = &'a Connection;

    fn get(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<Option<Self::Entity>> {
        let query = r#"
            SELECT
                url,
                name,
                summary,
                category_id
            FROM
                thing
            WHERE
                url = $1
            "#;

        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map([entity_id], |row| {
            let url: Self::EntityId = row.get(0)?;
            let name: String = row.get(1)?;
            let summary: Option<String> = row.get(2)?;
            let category_id: tag::Id = row.get(3)?;

            Ok(Self::Entity::new(url, name, summary, category_id))
        })?;

        match rows.next() {
            Some(value) => Ok(Some(value?)),
            None => Ok(None),
        }
    }

    fn to_vec(conn: Self::Conn) -> Result<Vec<Self::Entity>> {
        let query = r#"
            SELECT
                url,
                name,
                summary,
                category_id
            FROM
                thing
            "#;

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let url: Self::EntityId = row.get(0)?;
            let name: String = row.get(1)?;
            let summary: Option<String> = row.get(2)?;
            let category_id: tag::Id = row.get(3)?;

            Ok(Self::Entity::new(url, name, summary, category_id))
        })?;
        let mut items = Vec::new();

        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    fn len(conn: Self::Conn) -> Result<usize> {
        let query = r#"
            SELECT
                count(1)
            FROM
                thing
            "#;
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map([], |row| {
            let count: usize = row.get(0)?;

            Ok(count)
        })?;

        match rows.next() {
            Some(value) => Ok(value?),
            None => unreachable!(),
        }
    }

    fn add(conn: Self::Conn, entity: &Self::Entity) -> Result<()> {
        let record = params![
            entity.url(),
            entity.name(),
            entity.summary(),
            entity.category_id(),
        ];

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO staging.thing
                (url, name, summary, category_id)
            VALUES
                (?, ?, ?, ?)
            "#,
        )?;

        stmt.execute(record)?;

        Ok(())
    }

    // TODO: With the split of source and staging removing an item is non trivial.
    fn remove(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<()> {
        let query = r#"
            DELETE
            FROM
                staging.thing
            WHERE
                url = $1
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute([entity_id])?;

        // Remove relations.
        let query_rel = r#"
            DELETE
            FROM
                staging.thing_tag
            WHERE
                thing_id = $1
            "#;
        let mut stmt_rel = conn.prepare(query_rel)?;
        stmt_rel.execute([entity_id])?;

        Ok(())
    }

    fn replace(conn: Self::Conn, entity: &Self::Entity) -> Result<()> {
        let record = params![
            entity.url(),
            entity.name(),
            entity.summary(),
            entity.category_id(),
        ];

        let mut stmt = conn.prepare(
            r#"
            INSERT OR REPLACE INTO staging.thing
                (url, name, summary, category_id)
            VALUES
                (?, ?, ?, ?)
            "#,
        )?;

        stmt.execute(record)?;

        Ok(())
    }
}

impl ThingStore {
    pub fn list_categorised<Conn>(conn: Conn, category_id: &tag::Id) -> Result<Vec<thing::Thing>>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = r#"
            SELECT
                url,
                name,
                summary,
                category_id
            FROM
                thing
            WHERE
                category_id = $1
            "#;

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([category_id], |row| {
            let url: thing::Id = row.get(0)?;
            let name: String = row.get(1)?;
            let summary: Option<String> = row.get(2)?;
            let category: tag::Id = row.get(3)?;
            // TODO: propagate error rather than excepting.
            let tags = tags_for(&conn, &url).expect("Failed to fetch tags.");

            Ok(thing::Thing {url, name, summary, category, tags})
        })?;
        let mut items = Vec::new();

        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }
}

fn tags_for<Conn>(conn: &Conn, thing_id: &thing::Id) -> Result<Vec<tag::Id>>
where
    Conn: Deref<Target = Connection>,
{
    let query = r#"
        SELECT
            tag_id
        FROM
            source.thing_tag
        WHERE
            thing_id = $1
        ORDER BY tag_id
    "#;

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([thing_id], |row| {
        let id: String = row.get(0)?;

        Ok(id)
    })?;
    let mut items = Vec::new();

    for row in rows {
        items.push(row?);
    }

    Ok(items)
}
