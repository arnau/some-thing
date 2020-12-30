use clap::Clap;
use std::fs::{remove_dir, remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use super::Prompter;
use crate::package::resource::Resource;
use crate::package::{self, Package};
use crate::{Event, Report, Result, SomeError};

/// Destroy a Some package in an existing directory.
///
/// This command will destroy any file and directory that is part of the Some package definition.
/// That is:
///
/// * `datapackage.json`
/// * `data/thing.csv`
/// * `data/tag.csv`
/// * `data/thing_tag.csv`
/// * `data/` if after removing all the above it is empty
#[derive(Debug, Clap)]
pub struct Cmd {
    /// The location where to find the Some package to be destroyed.
    #[clap(default_value = ".")]
    path: PathBuf,
    /// Run without any user confirmation.
    #[clap(long = "force")]
    force_flag: bool,
}

impl Cmd {
    pub fn run(&self) -> Result<Report> {
        let mut prompter = Prompter::new()?;
        let full_path = &self.path.canonicalize()?;
        let package_file = File::open(package::DESCRIPTOR_PATH)
            .map_err(|_| SomeError::MissingPackageDescriptor(full_path.display().to_string()))?;
        let package_reader = BufReader::new(package_file);
        let package: Package = serde_json::from_reader(package_reader)?;
        let package_name = package.name().to_string();

        if !self.force_flag {
            let name = prompter.demand("Confirm the name of the package to destroy")?;

            if name != package_name {
                return Err(SomeError::SealError(format!(
                    "The given name `{}` does not match the data package `{}`.",
                    name, package_name
                )));
            }
        }

        for resource in package.resources {
            remove_resource(&self.path, &resource)?;
        }

        if remove_dir(&self.path.join(package::DATA_PATH)).is_err() {
            Event::new("Keeping the data directory as it is not empty.");
        }

        remove_file(&self.path.join(package::DESCRIPTOR_PATH))?;

        prompter.flush()?;

        let report = Report::new(format!("Package `{}` destroyed.", &package_name));
        Ok(report)
    }
}

fn remove_resource<P: AsRef<Path>>(path: P, resource: &Resource) -> Result<()> {
    let path = path.as_ref().join(resource.path());
    remove_file(&path)?;

    Ok(())
}
