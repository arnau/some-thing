use crate::tag::TagId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{fmt, io};

#[derive(Debug)]
pub struct Thing {
    url: String,
    name: String,
    tags: Vec<TagId>,
    summary: Option<String>,
}

impl Thing {
    pub fn new(url: String, name: String, tags: Vec<TagId>, summary: Option<String>) -> Self {
        Self {
            url,
            name,
            tags,
            summary,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tags(&self) -> &Vec<TagId> {
        &self.tags
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

#[derive(Debug)]
pub struct NewThing {
    url: String,
    name: String,
    tags: Vec<String>,
    category_id: String,
    summary: Option<String>,
}

impl NewThing {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn category_id(&self) -> &str {
        &self.category_id
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NewThingBuilder {
    #[serde(with = "empty_string")]
    url: Option<String>,
    #[serde(with = "empty_string")]
    name: Option<String>,
    summary: Option<String>,
    tags: Vec<String>,
    #[serde(with = "empty_string")]
    category_id: Option<String>,
}

impl NewThingBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            name: None,
            summary: None,
            tags: Vec::new(),
            category_id: None,
        }
    }

    pub fn with_url<S: Into<String>>(&mut self, url: S) -> &Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_name<S: Into<String>>(&mut self, name: S) -> &Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_summary<S: Into<String>>(&mut self, summary: S) -> &Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_tags(&mut self, tags: &[String]) -> &Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_tag<S: Into<String>>(&mut self, tag: S) -> &Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_category_id<S: Into<String>>(&mut self, category_id: S) -> &Self {
        self.category_id = Some(category_id.into());
        self
    }

    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    pub fn category_id(&self) -> Option<&String> {
        self.category_id.as_ref()
    }

    /// Builds the actual `Record`.
    ///
    /// Notice that it consumes the builder.
    ///
    /// ## Examples
    ///
    /// ```
    /// use curator_sketch::history::RecordBuilder;
    ///
    /// let b = RecordBuilder::new("https://www.seachess.net")
    ///     .with_title("Seachess")
    ///     .with_summary("A summary")
    ///     .with_tags(&vec!["a", "b", "c"])
    ///     .build();
    ///
    /// assert!(b.is_ok(), "Expected the record to build correctly");
    /// ```
    pub fn build(self) -> Result<NewThing, NewThingError> {
        let record = NewThing {
            url: self.url.ok_or(NewThingError::MissingUrl)?,
            name: self.name.ok_or(NewThingError::MissingName)?,
            summary: self.summary,
            category_id: self.category_id.ok_or(NewThingError::MissingCategory)?,
            tags: self.tags,
        };

        Ok(record)
    }
}

mod empty_string {
    use serde::Deserialize;

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *value {
            Some(ref v) => serializer.serialize_some(v),
            None => serializer.serialize_some(""),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let opt = if value.trim().is_empty() {
            None
        } else {
            Some(value.trim().to_string())
        };

        Ok(opt)
    }
}

#[derive(Debug)]
pub enum NewThingError {
    MissingUrl,
    MissingName,
    MissingCategory,
    Csv(csv::Error),
    Io(io::Error),
}

impl fmt::Display for NewThingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NewThingError::*;

        match self {
            MissingUrl => write!(f, "'url' is a required field"),
            MissingName => write!(f, "'name' is a required field"),
            MissingCategory => write!(f, "'category_id' is a required field"),
            Csv(err) => write!(f, "{}", err),
            Io(err) => write!(f, "{}", err),
        }
    }
}

impl Error for NewThingError {}

impl From<io::Error> for NewThingError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<csv::Error> for NewThingError {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err)
    }
}
