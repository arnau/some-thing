use std::ops::Deref;

use crate::{
    entities::change::{Change, Event, EventId},
    Result,
};

use super::{params, Connection};

#[derive(Debug)]
pub struct ChangeStore;

impl ChangeStore {
    pub fn get<Conn>(conn: &Conn, event_id: EventId) -> Result<Option<Change>>
    where
        Conn: Deref<Target = Connection>,
    {
        let record = params![
            &event_id.timestamp,
            &event_id.operation,
            &event_id.kind,
            &event_id.id,
        ];

        let query = r#"
            SELECT
                data
            FROM
                staging.changelog
            WHERE
                timestamp = $1
            AND
                operation = $2
            AND
                kind = $3
            AND
                id = $3
            "#;
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map(record, |row| {
            let raw: String = row.get(0)?;

            Ok(raw)
        })?;

        match rows.next() {
            Some(value) => {
                let data: Change = serde_json::from_str(&value?)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    pub fn to_vec<Conn>(conn: &Conn) -> Result<Vec<Event>>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = r#"
            SELECT
                unixepoch(timestamp),
                data
            FROM
                staging.changelog
            ORDER BY timestamp ASC
            "#;

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let ts: usize = row.get(0)?;
            let raw: String = row.get(1)?;

            Ok((ts, raw))
        })?;
        let mut items = Vec::new();

        for row in rows {
            let row = row?;
            let timestamp = row.0;
            let change: Change = serde_json::from_str(&row.1)?;
            items.push(Event { timestamp, change });
        }

        Ok(items)
    }

    pub fn len<Conn>(conn: &Conn) -> Result<usize>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = r#"
            SELECT
                count(1)
            FROM
                staging.changelog
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

    pub fn add<Conn>(conn: &Conn, entity: &Change) -> Result<()>
    where
        Conn: Deref<Target = Connection>,
    {
        let record = serde_json::to_string(entity)?;

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO staging.changelog
                (data)
            VALUES
                ($1)
            "#,
        )?;

        stmt.execute([record])?;

        Ok(())
    }

    pub fn remove<Conn>(conn: &Conn, event_id: EventId) -> Result<()>
    where
        Conn: Deref<Target = Connection>,
    {
        let record = params![
            &event_id.timestamp,
            &event_id.operation,
            &event_id.kind,
            &event_id.id,
        ];

        let query = r#"
            DELETE
            FROM
                staging.changelog
            WHERE
                timestamp = $1
            AND
                operation = $2
            AND
                kind = $3
            AND
                id = $3
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute(record)?;

        Ok(())
    }

    pub fn flush<Conn>(conn: &Conn) -> Result<()>
    where
        Conn: Deref<Target = Connection>,
    {
        let query = r#"
            DELETE
            FROM
                staging.changelog
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute([])?;

        Ok(())
    }
}
