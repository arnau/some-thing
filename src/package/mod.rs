//! This module implements a naive version of the [Tabular Data Package] specification. This
//! implementation has stronger requirements than the specification, in particular it expects the
//! following fields to be present:
//!
//! * `id`
//! * `name`
//! * `title`
//! * `description`
//! * `created`
//!
//! The creation of a package requires quite a few bits of information so the [`PackageBuilder`]
//! implements the builder pattern to ease the task. Note that to build a valid [`Package`] you'll
//! need to create resources either using the [`ResourceBuilder`][resource::ResourceBuilder] or a helper function
//! such as [`thing::package_resource`][crate::lenses::thing::package_resource] for the
//! [`Thing`][crate::thing::Thing] entity.
//!
//! ## Examples
//!
//! The following example creates a package for a collection of SQLite resources. Notice that the
//! name provided must obey some restrictive rules to be a valid. Check the [`Name`] definition for
//! the specifics.
//!
//! ```
//! use some::{
//!     package::PackageBuilder,
//!     package::core::Name,
//!     lenses::thing::package_resource,
//! };
//!
//! let package = PackageBuilder::new("some-sqlite")
//!     .expect("to be a package builder with a safe name")
//!     .title("Some SQLite")
//!     .description("A collection of some SQLite resources.")
//!     .resources(vec![package_resource()])
//!     .build();
//!
//! assert!(package.is_ok());
//! ```
//!
//! More times than not, you'll have already a `datapackage.json` file in place so the Serde
//! implementation will be convenient:
//!
//! ```no_run
//! use serde_json;
//! use some::package::Package;
//!
//! let package: Result<Package, _> = serde_json::from_str(r#"{ ... }"#);
//! ```
//!
//! [Tabular Data Package]: https://specs.frictionlessdata.io/tabular-data-package/

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

pub mod core;
pub mod resource;

use self::core::*;
use self::resource::Resource;

pub const DESCRIPTOR_PATH: &str = "datapackage.json";
pub const DATA_PATH: &str = "data/";

/// Represents a Tabular Data Package.
///
/// Use either the [`PackageBuilder`] or `serde_json::from_str` to create a new `Package`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub profile: PackageProfile,
    pub id: Identifier,
    pub name: Name,
    pub title: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub resources: Vec<Resource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<Licence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<Url>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<Contributor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

impl Package {
    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// The main way to build a `Package`.
///
/// ## Examples
///
/// ```
/// use some::{
///     package::PackageBuilder,
///     package::core::Name,
///     lenses::thing::package_resource,
/// };
/// use std::str::FromStr;
///
/// let package = PackageBuilder::new("some-sqlite")
///     .expect("to be a package builder with a safe name")
///     .title("Some SQLite")
///     .description("A collection of some SQLite resources.")
///     .resources(vec![package_resource()])
///     .build();
///
/// assert!(package.is_ok());
/// ```
#[derive(Debug, Default)]
pub struct PackageBuilder {
    id: Option<Identifier>,
    name: Option<Name>,
    title: Option<String>,
    description: Option<String>,
    licenses: Vec<Licence>,
    homepage: Option<Url>,
    resources: Vec<Resource>,
    created: Option<DateTime<Utc>>,
    contributors: Vec<Contributor>,
    keywords: Vec<String>,
}

impl PackageBuilder {
    /// Creates a new `PackageBuilder` with the given name.
    ///
    /// ## Errors
    ///
    /// * Returns [`PackageError::MalformedName`] if the given string is not a valid [`Name`].
    pub fn new(value: &str) -> Result<Self, PackageError> {
        let mut builder = Self::default();
        builder.name = Some(Name::from_str(value)?);

        Ok(builder)
    }

    /// Sets a custom identifier.
    ///
    /// When not informed, the [`PackageBuilder::build`] method will attempt to create a UUIDv4 for you.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use some::package::PackageBuilder;
    /// PackageBuilder::new("some-rust").unwrap()
    ///     .id("6764DF8B-9CCB-4446-8A61-7C7611DCEDD2");
    /// ```
    pub fn id<V: Into<Identifier>>(mut self, value: V) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Sets a human readable identifier.
    ///
    /// Typically you'd want to provide the name using the [`PackageBuilder::new`] method but this
    /// might be convenient if you want to circumvent the validation checks for [`Name::from_str`].
    ///
    /// ## Examples
    ///
    /// ```
    /// # use some::package::PackageBuilder;
    /// # use some::package::core::Name;
    /// PackageBuilder::default().name(Name::new("some-rust"));
    /// ```
    pub fn name(mut self, value: Name) -> Self {
        self.name = Some(value);
        self
    }

    pub fn title<V: Into<String>>(mut self, value: V) -> Self {
        self.title = Some(value.into());
        self
    }

    pub fn description<V: Into<String>>(mut self, value: V) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn licenses(mut self, value: Vec<Licence>) -> Self {
        self.licenses = value;
        self
    }

    pub fn homepage<V: Into<Url>>(mut self, value: V) -> Self {
        self.homepage = Some(value.into());
        self
    }

    pub fn resources(mut self, value: Vec<Resource>) -> Self {
        self.resources = value;
        self
    }

    pub fn timestamp(mut self, value: DateTime<Utc>) -> Self {
        self.created = Some(value);
        self
    }

    pub fn contributors(mut self, value: Vec<Contributor>) -> Self {
        self.contributors = value;
        self
    }

    pub fn keywords(mut self, value: Vec<String>) -> Self {
        self.keywords = value;
        self
    }

    pub fn build(self) -> Result<Package, PackageError> {
        let name = if let Some(value) = self.name {
            value.clone()
        } else {
            return Err(PackageError::RequiredField("name".into()));
        };

        let title = if let Some(value) = self.title {
            value
        } else {
            return Err(PackageError::RequiredField("title".into()));
        };

        let description = if let Some(value) = self.description {
            value
        } else {
            return Err(PackageError::RequiredField("description".into()));
        };

        if self.resources.is_empty() {
            return Err(PackageError::RequiredField("resources".into()));
        };

        Ok(Package {
            profile: Profile::new("tabular-data-package"),
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name,
            title,
            description,
            licenses: self.licenses,
            homepage: self.homepage,
            resources: self.resources,
            created: self.created.unwrap_or_else(|| Utc::now()),
            contributors: self.contributors,
            keywords: self.keywords,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical() -> String {
        r#"{
          "profile": "tabular-data-package",
          "id": "00000000-0000-0000-0000-000000000000",
          "name": "some-nothingness",
          "title": "Some Nothingness",
          "description": "Nothing",
          "licenses": [{
            "name": "ODC-PDDL",
            "path": "http://opendatacommons.org/licenses/pddl/",
            "title": "Open Data Commons Public Domain Dedication and License"
          }],
          "homepage": "https://github.com/arnau/some-nothingness",
          "created": "2020-12-29T10:11:12Z",
          "contributors": [{
            "title": "Arnau Siches",
            "path": "https://www.seachess.net/",
            "role": "author"
          }],
          "keywords": [
            "sqlite",
            "some-thing"
          ],
          "resources": [
            {
              "profile": "tabular-data-resource",
              "name": "thing",
              "title": "Thing",
              "description": "The set of things for the collection.",
              "path": "data/thing.csv",
              "encoding": "UTF-8",
              "schema": {
                "fields": [
                  {
                    "name": "url",
                    "description": "The URL of the thing.",
                    "type": "string",
                    "format": "uri",
                    "constraints": [{
                      "required": true,
                      "unique": true
                    }]
                  },
                  {
                    "name": "name",
                    "description": "The name of the thing.",
                    "type": "string",
                    "constraints": [{
                      "required": true,
                      "unique": true
                    }]
                  },
                  {
                    "name": "summary",
                    "description": "The description of the thing.",
                    "type": "string",
                    "constraints": [{
                      "required": false,
                      "unique": false
                    }]
                  },
                  {
                    "name": "category_id",
                    "description": "The category of the thing.",
                    "type": "string",
                    "constraints": [{
                      "required": true,
                      "unique": false
                    }]
                  }
                ],
                "primaryKey": ["url"],
                "foreignKeys": [{
                  "fields": ["category_id"],
                  "reference": {
                    "resource": "tag",
                    "fields": ["id"]
                  }
                }]
              }
            },
            {
              "profile": "tabular-data-resource",
              "name": "tag",
              "title": "Tag",
              "description": "The set of tags to classify the collection of things.",
              "path": "data/tag.csv",
              "encoding": "UTF-8",
              "schema": {
                "fields": [
                  {
                    "name": "id",
                    "description": "The tag identifier.",
                    "type": "string",
                    "constraints": [{
                      "required": true,
                      "unique": true
                    }]
                  },
                  {
                    "name": "name",
                    "description": "The tag name.",
                    "type": "string",
                    "constraints": [{
                      "required": true,
                      "unique": true
                    }]
                  },
                  {
                    "name": "summary",
                    "description": "The tag description.",
                    "type": "string",
                    "constraints": [{
                      "required": false,
                      "unique": false
                    }]
                  }
                ],
                "primaryKey": ["id"]
              }
            },
            {
              "profile": "tabular-data-resource",
              "name": "thing_tag",
              "title": "Thing tags",
              "description": "The set of tags to further classify the collection of things.",
              "path": "data/thing_tag.csv",
              "encoding": "UTF-8",
              "schema": {
                "fields": [
                  {
                    "name": "thing_id",
                    "description": "The reference to a thing.",
                    "type": "string",
                    "format": "uri",
                    "constraints": [{
                      "required": true,
                      "unique": false
                    }]
                  },
                  {
                    "name": "tag_id",
                    "description": "The reference to a tag.",
                    "type": "string",
                    "constraints": [{
                      "required": true,
                      "unique": false
                    }]
                  }
                ],
                "primaryKey": ["thing_id", "tag_id"],
                "foreignKeys": [{
                  "fields": ["thing_id"],
                  "reference": {
                    "resource": "thing",
                    "fields": ["url"]
                  }
                }, {
                "fields": ["tag_id"],
                "reference": {
                  "resource": "tag",
                  "fields": ["id"]
                }
                }]
              }
            }
          ]
        }"#
        .to_string()
    }

    #[test]
    fn deserialise() {
        let raw = canonical();
        let actual: Result<Package, _> = serde_json::from_str(&raw);
        assert!(actual.is_ok());
    }

    #[test]
    fn fullround() -> Result<(), Box<dyn std::error::Error>> {
        let raw = canonical();
        let pkg: Package = serde_json::from_str(&raw)?;
        let actual = serde_json::to_string(&pkg);

        assert!(actual.is_ok());

        Ok(())
    }
}
