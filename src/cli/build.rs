use clap::Parser;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::context::Context;
use crate::store::{TagStore, ThingStore};
use crate::entities::thing;
use crate::{Report, Result};

/// Builds the Markdown version of the collection.
#[derive(Debug, Parser)]
pub struct Cmd {
    // /// The path to the cache.
    // #[clap(long, value_name = "path", default_value = DEFAULT_PATH)]
    // cache: Strategy,
    /// Flag to use the README.md found in the given path.
    #[clap(short, action, default_value_t = false)]
    output_flag: bool,
    /// The location where to find the Some package to be destroyed.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut context = Context::new(&self.path)?;
        let mut writer: Box<dyn Write> = if self.output_flag {
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(self.path.join("README.md"))?;

            Box::new(file)
        } else {
            Box::new(io::stdout())
        };

        write_readme(&mut context, &mut writer)?;

        Ok(Report::new(""))
    }
}

fn write_readme<W: Write + ?Sized>(context: &mut Context, mut writer: &mut W) -> Result<()> {
    write_header(context, &mut writer)?;
    write_body(context, &mut writer)?;
    write_footer(context, &mut writer)?;

    Ok(())
}

fn write_header<W: Write>(context: &mut Context, writer: &mut W) -> Result<()> {
    let package = context.package();
    writeln!(writer, "# {}\n", package.title())?;
    writeln!(writer, "{}\n", package.description())?;

    Ok(())
}

fn write_footer<W: Write>(context: &mut Context, writer: &mut W) -> Result<()> {
    let package = context.package();

    if !package.licenses().is_empty() {
        writeln!(writer, "\n## Licence\n")?;

        for licence in package.licenses() {
            writeln!(
                writer,
                "This dataset is licensed under the [{}]({}).\n",
                licence.title(),
                licence.path(),
            )?;
        }
    }

    Ok(())
}

fn write_body<W: Write>(context: &mut Context, writer: &mut W) -> Result<()> {
    let store = context.store();
    let categories = TagStore::list_categories(&store.conn)?;

    if categories.is_empty() {
        writeln!(writer, "**This collection is empty**")?;

        return Ok(());
    }

    for category in categories {
        let things = ThingStore::list_categorised(&store.conn, &category.id())?;
        writeln!(
            writer,
            "\n## {}\n",
            category.name().unwrap_or(category.id())
        )?;

        if let Some(summary) = category.summary() {
            writeln!(writer, "{}\n", summary)?;
        }

        write_table(writer, &things)?;
    }

    Ok(())
}

fn write_table<W: Write>(writer: &mut W, things: &Vec<thing::Thing>) -> Result<()> {
    writeln!(writer, "| name | summary | tags |")?;
    writeln!(writer, "| - | - | - |")?;

    for thing in things {
        write_row(writer, thing)?;
    }

    Ok(())
}

fn write_row<W: Write>(writer: &mut W, thing: &thing::Thing) -> Result<()> {
    let link = format!("[{}]({})", &thing.name, &thing.url);
    let summary = &thing.summary;

    writeln!(
        writer,
        "| {} | {} | {} |",
        link,
        summary.as_ref().unwrap_or(&"".to_string()),
        thing.tags.join("; ")
    )?;

    Ok(())
}
