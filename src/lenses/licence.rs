use serde::{Deserialize, Serialize};
use skim::prelude::*;
use std::fmt;
use std::iter::FromIterator;

use crate::package::core::Licence;
use crate::Result;

pub fn fetch_licenses() -> Result<Vec<Licence>> {
    let mut set: Vec<Licence> = Vec::new();

    let body =
        reqwest::blocking::get("https://licenses.opendefinition.org/licenses/groups/ckan.json")?
            .json::<Vec<FullLicence>>()?;

    for full_licence in body {
        set.push(full_licence.into());
    }

    Ok(set)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullLicence {
    pub id: String,
    pub domain_content: bool,
    pub domain_data: bool,
    pub domain_software: bool,
    pub od_conformance: String,
    pub osd_conformance: String,
    pub status: String,
    pub title: String,
    pub url: String,
}

impl From<FullLicence> for Licence {
    fn from(full: FullLicence) -> Self {
        Licence {
            name: full.id,
            path: full.url,
            title: full.title,
        }
    }
}

impl SkimItem for Licence {
    fn display(&self, _: DisplayContext) -> AnsiString {
        self.name.clone().into()
    }

    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    fn preview(&self, _: PreviewContext) -> ItemPreview {
        ItemPreview::Text(format!("[{}]({})\n{}", self.name, self.path, self.title))
    }
}

#[derive(Debug, Clone)]
pub struct LicenceSet(Vec<Licence>);

impl LicenceSet {
    pub fn new(raw: Vec<Licence>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[Licence] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Licence> {
        self.0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Licence> {
        self.0.first()
    }
}

impl IntoIterator for LicenceSet {
    type Item = Licence;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Licence> for LicenceSet {
    fn from_iter<I: IntoIterator<Item = Licence>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        Self::new(v)
    }
}

impl fmt::Display for LicenceSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list: Vec<String> = self.0.iter().map(|item| format!("{}", item)).collect();

        if f.alternate() {
            write!(f, "{}", list.join("\n"))
        } else {
            write!(f, "{}", list.join(";"))
        }
    }
}

impl LicenceSet {
    pub fn as_skim_buffer(&self) -> SkimItemReceiver {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        for tag in self.to_vec() {
            let _ = tx_item.send(Arc::new(tag));
        }

        drop(tx_item);

        rx_item
    }
}

impl From<LicenceSet> for SkimItemReceiver {
    fn from(input: LicenceSet) -> SkimItemReceiver {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        for tag in input {
            let _ = tx_item.send(Arc::new(tag));
        }

        drop(tx_item);

        rx_item
    }
}
