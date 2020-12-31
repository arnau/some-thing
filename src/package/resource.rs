use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::core::{Name, Profile, ResourceProfile};

/// Represents a [Tabular Data Resource](https://specs.frictionlessdata.io/tabular-data-resource/).
#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub profile: ResourceProfile,
    pub name: Name,
    pub title: String,
    pub description: String,
    pub path: PathBuf,
    pub encoding: Encoding,
    // pub bytes: u64,
    // pub hash: String,
    pub schema: Schema,
    // dialect: Dialect,
}

impl Resource {
    pub fn id(&self) -> &Name {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn field_names(&self) -> Vec<Name> {
        self.schema
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
    #[serde(rename = "primaryKey")]
    pub primary_key: Vec<Name>,
    #[serde(rename = "foreignKeys", default)]
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: Name,
    pub description: String,
    #[serde(rename = "type")]
    pub datatype: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub required: bool,
    pub unique: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForeignKey {
    pub fields: Vec<Name>,
    pub reference: Reference,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    pub resource: Name,
    pub fields: Vec<Name>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Encoding {
    #[serde(rename = "UTF-8")]
    Utf8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ResourceBuilder {
    name: Option<Name>,
    title: Option<String>,
    description: Option<String>,
    path: Option<String>,
    // bytes: u64,
    // hash: String,
    schema: Option<Schema>,
    // dialect: Dialect,
}

impl ResourceBuilder {
    pub fn new() -> Self {
        ResourceBuilder::default()
    }

    pub fn build(self) -> Resource {
        Resource {
            profile: Profile::new("tabular-data-resource"),
            name: self.name.expect("to be present."),
            title: self.title.expect("to be present."),
            description: self.description.expect("to be present."),
            path: self.path.expect("to be present.").into(),
            encoding: Encoding::Utf8,
            // bytes: u64,
            // hash: String,
            schema: self.schema.expect("to be present."),
            // dialect: Dialect,
        }
    }

    pub fn with_name(&mut self, value: Name) -> &Self {
        self.name = Some(value);
        self
    }

    pub fn with_title<S: Into<String>>(&mut self, value: S) -> &Self {
        self.title = Some(value.into());
        self
    }

    pub fn with_description<S: Into<String>>(&mut self, value: S) -> &Self {
        self.description = Some(value.into());
        self
    }

    pub fn with_path<S: Into<String>>(&mut self, value: S) -> &Self {
        self.path = Some(value.into());
        self
    }

    pub fn with_schema(&mut self, value: Schema) -> &Self {
        self.schema = Some(value);
        self
    }
}

// #[derive(Debug)]
// pub struct Dialect {
//     delimiter: String,
//     lineTerminator: String,
//     quoteChar: char,
// }
//
//       "dialect": {
//         "delimiter": ",",
//         "doubleQuote": true,
//         "header": true,
//         "lineTerminator": "\n",
//         "quoteChar": "\"",
//         "skipInitialSpace": false
//       },
