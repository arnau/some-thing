use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rusqlite::Transaction;

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
    store: Store,
}

impl Context {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let package = Package::from_path(&path)?;
        let strategy = Strategy::from_str(DEFAULT_PATH)?;
        let store = Store::open(path.to_path_buf(), &strategy)?;

        Ok(Self {
            package,
            path,
            store,
        })
    }

    pub fn store(&mut self) -> &mut Store {
        &mut self.store
    }

    pub fn tx(&mut self) -> Result<Transaction> {
        Ok(self.store.transaction()?)
    }

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
