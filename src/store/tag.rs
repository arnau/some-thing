use crate::{Result, tag::{Tag, TagId}};

use super::{Repository, Connection, params};

#[derive(Debug)]
pub struct TagStore;

impl<'a> Repository<'a> for TagStore {
    type Entity = Tag;
    type EntityId = TagId;
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
            let id: TagId = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(Tag::new(id, name, summary))
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
            let id: TagId = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let summary: Option<String> = row.get(2)?;

            Ok(Tag::new(id, name, summary))
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
        let record = params![
            entity.id(),
            entity.name(),
            entity.summary(),
        ];

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
        let record = params![
            entity.id(),
            entity.name(),
            entity.summary(),
        ];

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
