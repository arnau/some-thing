use std::ops::Deref;

use crate::{tag, Result};

use super::{params, Connection, Repository};

#[derive(Debug)]
pub struct TagStore;

impl<'a> Repository<'a> for TagStore {
    type Entity = tag::Record;
    type EntityId = tag::Id;
    type Conn = &'a Connection;

    fn get(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<Option<Self::Entity>> {
        let query = r#"
            SELECT
                id,
                name,
                summary
            FROM
                tag
            WHERE
                id = $1
            "#;

        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map([entity_id], |row| {
            let id: tag::Id = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(tag::Record::new(id, name, summary))
        })?;

        match rows.next() {
            Some(value) => Ok(Some(value?)),
            None => Ok(None),
        }
    }

    fn to_vec(conn: Self::Conn) -> Result<Vec<Self::Entity>> {
        let query = r#"
            SELECT
                id,
                name,
                summary
            FROM
                tag
            "#;

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let id: tag::Id = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(tag::Record::new(id, name, summary))
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
                tag
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
        let record = params![entity.id(), entity.name(), entity.summary(),];

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO staging.tag
                (id, name, summary)
            VALUES
                (?, ?, ?)
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
                staging.tag
            WHERE
                id = $1
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute([entity_id])?;

        // Remove relations.
        let query_rel = r#"
            DELETE
            FROM
                staging.thing_tag
            WHERE
                tag_id = $1
            "#;
        let mut stmt_rel = conn.prepare(query_rel)?;
        stmt_rel.execute([entity_id])?;

        Ok(())
    }

    fn replace(conn: Self::Conn, entity: &Self::Entity) -> Result<()> {
        let record = params![entity.id(), entity.name(), entity.summary(),];

        let mut stmt = conn.prepare(
            r#"
            INSERT OR REPLACE INTO staging.tag
                (id, name, summary)
            VALUES
                (?, ?, ?)
            "#,
        )?;

        stmt.execute(record)?;

        Ok(())
    }
}

impl TagStore {
    pub fn list<Conn>(conn: Conn) -> Result<Vec<tag::Record>>
    where
        Conn: Deref<Target = Connection>,
    {
        TagStore::to_vec(&conn)
    }

    pub fn list_without<Conn>(conn: Conn, ids: &[tag::Id]) -> Result<Vec<tag::Record>>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = format!(
            r#"
            SELECT
                id,
                name,
                summary
            FROM
                tag
            WHERE
                id NOT IN ({})
            "#,
            ids.iter()
                .map(|x| format!("'{}'", x))
                .collect::<Vec<String>>()
                .join(",")
        );

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let id: tag::Id = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(tag::Record::new(id, name, summary))
        })?;
        let mut items = Vec::new();

        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    pub fn list_categories<Conn>(conn: Conn) -> Result<Vec<tag::Record>>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = r#"
            SELECT DISTINCT
                tag.id,
                tag.name,
                tag.summary
            FROM
                source.thing AS thing
            LEFT JOIN
                source.tag AS tag ON thing.category_id = tag.id
            ORDER BY tag.id ASC
        "#;

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let id: tag::Id = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(tag::Record::new(id, name, summary))
        })?;
        let mut items = Vec::new();

        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }
}
