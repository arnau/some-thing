use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::package::core::Name;
use crate::package::resource::Resource;
use crate::package::Package;
use crate::store::{Store, Strategy, DEFAULT_PATH};
use crate::Result;

/// The holder of all contextual information.
#[derive(Debug)]
pub struct Context {
    /// The original location where the package was found.
    path: PathBuf,
    /// The package descriptor.
    package: Package,
    /// The cache store.
    cache: Store,
}

impl Context {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let package = Package::from_path(&path)?;
        let strategy = Strategy::from_str(DEFAULT_PATH)?;
        let cache = Store::open(path.to_path_buf(), &strategy)?;

        Ok(Self {
            package,
            path,
            cache,
        })
    }

    pub fn cache(&mut self) -> &mut Store {
        &mut self.cache
    }

    // TODO: Review
    // /// Writes the cache to disk.
    // pub fn flush(&self) -> Result<()> {
    //     // for resource in resources {
    //     //  take the name and query all
    //     //  open the file from path
    //     //  and write the result as CSV.

    //     for resource in self.resources() {
    //         let name = resource.id();
    //         let mut file = File::open(resource.path())?;
    //         let mut wtr = csv::Writer::from_writer(file);
    //         let rows = self.store.query("SELECT * FROM ?;", &[name], |row| {
    //             Ok()
    //         })
    //         // wtr.serialize(record)
    //     }

    //     // TODO: Write down package (if it has changed)
    //     // TODO: Write down README.
    //     Ok(())
    // }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn resources(&self) -> Vec<Resource> {
        self.package
            .resources()
            .iter()
            .map(|resource| {
                let mut new = resource.clone();
                new.path = self.path.join(resource.path());
                new
            })
            .collect()
    }

    pub fn open_resource(&self, name: &str) -> Result<File> {
        let name = Name::new(name);
        let resource: Resource = self
            .package
            .resources()
            .iter()
            .find(|r| r.id() == &name)
            .expect("resource to exist")
            .clone();

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .truncate(false)
            .open(self.path.join(resource.path()))?;

        Ok(file)
    }
}
