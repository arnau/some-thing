use clap::Parser;
use std::fs::{create_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::Prompter;
use crate::lenses;
use crate::lenses::licence::{fetch_licenses, LicenceSet};
use crate::package::core::{Licence, Name};
use crate::package::resource::Resource;
use crate::package::{self, Package, PackageBuilder};
use crate::{Report, Result};

/// Creates a new Some package in an existing directory.
///
/// This command will scaffold a new Some-flavoured [Tabular Data Package] in the current directory.
/// Give a path as an argument to create in the given directory.
///
/// [Tabular Data Package]: https://specs.frictionlessdata.io/tabular-data-package/
#[derive(Debug, Parser)]
pub struct Cmd {
    /// The location where to scaffold a new Some package.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut prompter = Prompter::new()?;

        // TODO: Nicely recover from a bad package name.
        // TODO: Provide hint for allowed chars.
        let raw_name = prompter.demand("package name")?;
        let name = Name::from_str(&raw_name)?;
        let title = prompter.demand("title")?;
        let description = prompter.demand("description")?;
        let homepage = prompter.ask_once("homepage (URL)")?;
        let resources = lenses::package::resources();
        let licenses: Vec<Licence> =
            if let Some(answer) = prompter.ask_once("do you want to add a licence? (y/N)")? {
                if answer == "y" {
                    let items = fetch_licenses()?;

                    let choices = prompter.read_choices(
                        items.clone().into_iter().collect::<LicenceSet>(),
                        "licenses",
                    )?;

                    let mut result: Vec<Licence> = Vec::new();
                    for licence in items {
                        if choices.iter().any(|name| name == &licence.name) {
                            result.push(licence);
                        }
                    }

                    result
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
        // let contributors: Vec<Contributor>,
        // let keywords: Vec<String>,

        let mut builder = PackageBuilder::default()
            .name(name)
            .title(title)
            .description(description)
            .licenses(licenses)
            .resources(resources);

        if let Some(value) = homepage {
            builder = builder.homepage(value);
        }

        let package = builder.build()?;

        // Write Package
        write_package(&self.path.join(package::DESCRIPTOR_PATH), &package)?;
        create_dir(&self.path.join(package::DATA_PATH))?;
        for resource in package.resources {
            write_resource(&self.path, &resource)?;
        }

        prompter.flush()?;

        let report = Report::new("Success");
        Ok(report)
    }
}

fn write_package<P: AsRef<Path>>(path: P, package: &Package) -> Result<()> {
    let s = serde_json::to_string_pretty(&package)?;
    let mut file = File::create(path)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}

fn write_resource<P: AsRef<Path>>(path: P, resource: &Resource) -> Result<()> {
    let path = path.as_ref().join(resource.path());
    let file = File::create(path)?;
    let mut wtr = csv::Writer::from_writer(&file);
    let field_names = resource.field_names();

    wtr.write_record(&field_names)?;

    // TODO: Find a better place for this. Some sort of `default_records` perhaps.
    if resource.id().to_string() == "tag" {
        wtr.write_record(&["miscellaneous", "Miscellaneous", "The unclassifiable."])?;
    }

    wtr.flush()?;

    Ok(())
}
