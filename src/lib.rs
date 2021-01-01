use chrono::format::ParseError as ChronoError;
use csv;
use rusqlite;
use rustyline::error::ReadlineError;
use serde::{de, ser};
use std::fmt::Display;
use std::{fmt, io, num};
use thiserror::Error;
use toml;
use url;

pub mod cli;
pub mod context;
pub mod lenses;
pub mod package;
pub mod store;
pub mod tag;
pub mod thing;

use package::core::PackageError;
use store::StoreError;
use thing::ThingError;

pub type Result<T> = std::result::Result<T, SomeError>;

/// A report for the user.
#[derive(Debug)]
pub struct Report {
    message: String,
}

impl Report {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// A report event.
#[derive(Debug, Clone)]
pub struct Event(String);

impl Event {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self(message.into())
    }
}

/// The main application error for some.
#[derive(Error, Debug)]
pub enum SomeError {
    // Internal
    #[error(transparent)]
    Thing(#[from] ThingError),
    #[error(transparent)]
    Package(#[from] PackageError),
    #[error(transparent)]
    StoreError(#[from] StoreError),

    #[error("unknown {0}")]
    Unknown(String),
    #[error("url exists '{0}'")]
    UrlExists(String),
    #[error("bad url '{0}'")]
    BadUrl(String),
    #[error("required field")]
    FieldRequired(String),
    #[error("couldn't find the project directory")]
    ProjectDir,
    #[error("{0}")]
    SealError(String),
    #[error("`{0}` is not a Some package.")]
    MissingPackageDescriptor(String),

    // External
    #[error("date error")]
    Date(#[from] ChronoError),
    #[error("url error")]
    Url(#[from] url::ParseError),
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("csv error")]
    Csv(#[from] csv::Error),
    #[error("sqlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("unexpected integer")]
    ParseInt(#[from] num::ParseIntError),
    #[error("toml error")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml error")]
    TomlSer(#[from] toml::ser::Error),
    #[error("serde {0}")]
    Serde(String),
    #[error("readline")]
    Readline(#[from] ReadlineError),
    #[error("reqwest")]
    Fetch(#[from] reqwest::Error),
    #[error("json")]
    Json(#[from] serde_json::Error),
}

impl ser::Error for SomeError {
    fn custom<T: Display>(msg: T) -> Self {
        SomeError::Serde(msg.to_string())
    }
}

impl de::Error for SomeError {
    fn custom<T: Display>(msg: T) -> Self {
        SomeError::Serde(msg.to_string())
    }
}
