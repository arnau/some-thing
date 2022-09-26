use super::{params, Connection, Repository};
use crate::entities::{tag, thing, thingtag::Thingtag};
use crate::Result;

#[derive(Debug)]
pub struct ThingtagStore;

// TODO: An aux table is not really a repository.
impl<'a> Repository<'a> for ThingtagStore {
    type Entity = Thingtag;
    type EntityId = (thing::Id, tag::Id);
    type Conn = &'a Connection;

    fn get(_conn: Self::Conn, _entity_id: &Self::EntityId) -> Result<Option<Self::Entity>> {
        unimplemented!()
    }

    fn to_vec(_conn: Self::Conn) -> Result<Vec<Self::Entity>> {
        unimplemented!()
    }

    fn len(conn: Self::Conn) -> Result<usize> {
        let query = r#"
            SELECT
                count(1)
            FROM
                thing_tag
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
        let record = params![entity.thing_id(), entity.tag_id(),];

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO staging.thing_tag
                (thing_id, tag_id)
            VALUES
                (?, ?)
            "#,
        )?;

        stmt.execute(record)?;

        Ok(())
    }

    // TODO: With the split of source and staging removing an item is non trivial.
    fn remove(conn: Self::Conn, entity_id: &Self::EntityId) -> Result<()> {
        let record = params![entity_id.0, entity_id.1,];

        let query = r#"
            DELETE
            FROM
                staging.thing_tag
            WHERE
                thing_id = $1
            AND
                tag_id = $2
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute(record)?;

        Ok(())
    }

    fn replace(_conn: Self::Conn, _entity: &Self::Entity) -> Result<()> {
        unimplemented!()
    }
}
