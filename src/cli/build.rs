use clap::Clap;
use itertools::Itertools;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::context::Context;
use crate::store::{Strategy, DEFAULT_PATH};
use crate::tag::TagId;
use crate::tag_set::TagSet;
use crate::thing::Thing;
use crate::thing_set::ThingSet;
use crate::thingtag_set::ThingtagSet;
use crate::{Report, Result, SomeError};

/// Builds the Markdown version of the collection.
#[derive(Debug, Clap)]
pub struct Cmd {
    /// The path to the cache.
    #[clap(long, value_name = "path", default_value = DEFAULT_PATH)]
    cache: Strategy,
    /// The location where to find the Some package to be destroyed.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut context = Context::new(&self.path)?;
        let mut writer = io::stdout();

        write_readme(&mut context, &mut writer)?;

        Ok(Report::new(""))
    }
}

fn write_readme<W: Write>(context: &mut Context, mut writer: &mut W) -> Result<()> {
    let mut thing_file = context.open_resource("thing")?;
    let mut tag_file = context.open_resource("tag")?;
    let mut thingtag_file = context.open_resource("thing_tag")?;

    let thingset = ThingSet::from_reader(&mut thing_file)?;
    let tagset = TagSet::from_reader(&mut tag_file)?;
    let thingtagset = ThingtagSet::from_reader(&mut thingtag_file)?;

    let groups: Groups = thingset
        .clone()
        .into_iter()
        .into_group_map_by(|thing| thing.category_id().clone());
    let tag_groups: TagGroups = thingtagset
        .clone()
        .into_iter()
        .map(|thingtag| {
            (
                thingtag.thing_id().to_string(),
                thingtag.tag_id().to_string(),
            )
        })
        .into_group_map_by(|tuple| tuple.0.to_string());

    write_header(&context, &mut writer)?;
    write_body(&context, &mut writer, &groups, &tag_groups, &tagset)?;
    write_footer(&context, &mut writer)?;

    Ok(())
}

fn write_header<W: Write>(context: &Context, writer: &mut W) -> Result<()> {
    let package = context.package();
    writeln!(writer, "# {}\n", package.title())?;
    writeln!(writer, "{}\n", package.description())?;

    Ok(())
}

fn write_footer<W: Write>(context: &Context, writer: &mut W) -> Result<()> {
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

type Groups = HashMap<String, Vec<Thing>>;
type TagGroups = HashMap<String, Vec<(String, TagId)>>;

fn write_body<W: Write>(
    _context: &Context,
    writer: &mut W,
    groups: &Groups,
    tag_groups: &TagGroups,
    tagset: &TagSet,
) -> Result<()> {
    if groups.is_empty() {
        writeln!(writer, "**This collection is empty**")?;
    }

    for (category_id, things) in itertools::sorted(groups) {
        let category = tagset
            .clone()
            .into_iter()
            .find(|tag| tag.id() == category_id)
            .ok_or(SomeError::Unknown(format!(
                "The tag `{}` is missing. Corrupted dataset.",
                &category_id
            )))?;

        writeln!(writer, "\n## {}\n", category.name().unwrap_or(&category_id))?;
        if let Some(summary) = category.summary() {
            writeln!(writer, "{}\n", summary)?;
        }

        writeln!(writer, "| name | summary | tags |")?;
        writeln!(writer, "| - | - | - |")?;

        for thing in things {
            let link = format!("[{}]({})", &thing.name(), &thing.url());
            let tags = tag_groups[thing.url()].iter().map(|(_, tag)| tag);

            let summary = &thing.summary();
            writeln!(
                writer,
                "| {} | {} | {} |",
                link,
                summary.as_ref().unwrap_or(&"".to_string()),
                itertools::sorted(tags).join("; ")
            )?;
        }
    }

    Ok(())
}
