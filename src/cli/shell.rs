use ansi_term::Colour::Red;
use clap::Parser;
use rusqlite::{Connection, Error as RusqliteError, Rows};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tabwriter::TabWriter;

use crate::context::Context;
use crate::shell::{Config, OutputMode};
use crate::{Report, Result, SomeError};

/// Starts a new interactive shell (repl-like).
#[derive(Debug, Parser)]
pub struct Cmd {
    /// The location where to find the Some package to be used.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut context = Context::new(&self.path)?;
        let store = context.store();

        let mut config = Config::default();
        let mut editor = Editor::<()>::new()?;
        let mut next_expression: String = String::new();

        loop {
            let prompt = if next_expression.is_empty() {
                ">> "
            } else {
                ".. "
            };
            let readline = editor.readline(prompt);
            match readline {
                // dot commands. E.g. `.mode tabbed`
                Ok(expr) if expr.starts_with(".") => {
                    editor.add_history_entry(&expr);

                    match process_dotcommand(&expr[1..], &mut config) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
                Ok(expr) if expr.trim().is_empty() => {}
                Ok(expr) if !expr.ends_with(";") => {
                    next_expression.push_str(&expr);
                    next_expression.push_str("\n ");
                }
                Ok(expr)
                    if next_expression.to_lowercase().starts_with("select")
                        || expr.to_lowercase().starts_with("select") =>
                {
                    let expr = expr.trim();

                    next_expression.push_str(expr);
                    editor.add_history_entry(&next_expression);

                    match process_query(&store.conn, &next_expression, &config) {
                        Ok(_) => {}
                        Err(SomeError::Sqlite(RusqliteError::SqlInputError {
                            msg,
                            sql,
                            offset,
                            ..
                        })) => {
                            display_error(msg, sql, offset as usize);
                        }
                        Err(SomeError::Sqlite(RusqliteError::SqliteFailure(_, msg))) => {
                            println!("{}", Red.paint(msg.unwrap_or("unknown error".to_string())));
                        }

                        Err(err) => {
                            println!("unhandled! {:?}", err);
                            println!("{:?}", &next_expression);
                        }
                    }

                    next_expression.clear();
                }
                Ok(expr) => {
                    let expr = expr.trim();

                    next_expression.push_str(expr);

                    editor.add_history_entry(&next_expression);

                    match process_expression(&store.conn, &next_expression) {
                        Ok(_) => {}
                        Err(SomeError::Sqlite(RusqliteError::SqlInputError {
                            msg,
                            sql,
                            offset,
                            ..
                        })) => {
                            display_error(msg, sql, offset as usize);
                        }
                        Err(SomeError::Sqlite(RusqliteError::SqliteFailure(_, msg))) => {
                            println!("{}", Red.paint(msg.unwrap_or("unknown error".to_string())));
                        }

                        Err(err) => {
                            eprintln!("unhandled! {:?}", err);
                        }
                    }

                    next_expression.clear();
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    next_expression.clear();
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(_) => println!("No input"),
            }
        }

        Ok(Report::new("Success"))
    }
}

#[inline]
fn process_expression<'a>(conn: &Connection, query: &str) -> Result<()> {
    let mut stmt = conn.prepare(query)?;
    let res = stmt.execute([])?;
    println!("{:?}", res);

    Ok(())
}

#[inline]
fn process_query(conn: &Connection, query: &str, config: &Config) -> Result<()> {
    let mut stmt = conn.prepare(query)?;

    let column_names = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    let rows = stmt.query([])?;

    config.output_mode.render(rows, column_names)?;

    Ok(())
}

impl OutputMode {
    fn render(self, rows: Rows, column_names: Vec<String>) -> Result<()> {
        match self {
            Self::Tabbed => display_tabbed(rows, column_names),
            Self::Table => display_table(rows, column_names),
            Self::Jsonline => display_jsonline(rows, column_names),
        }
    }
}

fn display_tabbed(mut rows: Rows, column_names: Vec<String>) -> Result<()> {
    let mut tw = TabWriter::new(stdout()).padding(2);
    tw.write(column_names.join("\t").as_bytes())?;
    tw.write("\n".as_bytes())?;

    while let Some(row) = rows.next()? {
        let tup: Vec<String> = column_names
            .iter()
            .enumerate()
            .map(|(idx, _name)| {
                let value: Option<String> = row.get(idx).unwrap();
                value.unwrap_or("".to_owned())
            })
            .collect();
        tw.write(tup.join("\t").as_bytes())?;
        tw.write("\n".as_bytes())?;
    }

    tw.flush()?;

    Ok(())
}

fn display_table(mut rows: Rows, column_names: Vec<String>) -> Result<()> {
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::*;

    let num = column_names.len();
    let mut table = Table::new();
    let headers: Vec<Cell> = column_names.iter().map(|c| Cell::new(c)).collect();

    table
        .load_preset(UTF8_FULL)
        .set_header(headers)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth);

    while let Some(row) = rows.next()? {
        let mut tup: Vec<Cell> = Vec::with_capacity(num);

        for idx in 0..num {
            let value: Option<String> = row.get(idx).unwrap();
            tup.push(Cell::new(value.unwrap_or("".to_owned())));
        }

        table.add_row(tup);
    }

    println!("{table}");

    Ok(())
}

fn display_jsonline(mut rows: Rows, column_names: Vec<String>) -> Result<()> {
    while let Some(row) = rows.next()? {
        // TODO: column names are unqualified so joins with name clashes won't be colleced
        // correctly using a HashMap.
        let tup: HashMap<String, Option<String>> = column_names
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let value: Option<String> = row.get(idx).unwrap();
                (name.clone(), value)
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&tup)?);
    }

    Ok(())
}

fn display_error(msg: String, query: String, offset: usize) {
    let (left, right) = query.split_at(offset);
    println!(
        "{}{}",
        left.replacen('\n', " ", 20),
        Red.paint(right.replacen('\n', "", 20))
    );
    println!("{}^", " ".repeat(offset));
    println!("{}{}", " ".repeat(offset), msg);
}

fn process_dotcommand(expr: &str, config: &mut Config) -> Result<()> {
    if let Some((command, value)) = expr.split_once(' ') {
        match command {
            "mode" => {
                process_dotmode(value, config)?;
            }
            _ => return Err(SomeError::from(anyhow::anyhow!("Unknown command"))),
        }
    } else {
        println!("A dot command requires a value.");
    };

    Ok(())
}

fn process_dotmode(expr: &str, config: &mut Config) -> Result<()> {
    if !expr.trim().is_empty() {
        let mode = OutputMode::from_str(expr)?;
        config.output_mode = mode;
    } else {
        println!("The command `.mode` requires a value.");
    };

    Ok(())
}
