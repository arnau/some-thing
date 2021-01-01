use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Represents a [Profile](https://specs.frictionlessdata.io/profiles/).
///
/// See the [registry](https://specs.frictionlessdata.io/schemas/registry.json) for more details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile(String);

impl Profile {
    /// Creates a profile from an unchecked string.
    ///
    /// Prefer `Profile::from_str` to ensure only valid characters are accepted.
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl FromStr for Profile {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tabular-data-package" | "tabular-data-resource" => Ok(Self(s.into())),
            _ => Err(PackageError::UnknownProfile(s.into())),
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO
pub type PackageProfile = Profile;

// TODO
pub type ResourceProfile = Profile;

/// A global identifier such as a UUID or DOI.
// TODO:
pub type Identifier = String;

/// An identifier string. Lower case characters with `.`, `_`, `-` and `/` are allowed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name(String);

impl Name {
    /// Creates a name from an unchecked string.
    ///
    /// Prefer `Name::from_str` to ensure only valid characters are accepted.
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }
}

impl FromStr for Name {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert_name(s)?;

        Ok(Self(s.into()))
    }
}

impl AsRef<[u8]> for Name {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Checks whether a string is a valid `Name`.
fn assert_name(name: &str) -> Result<(), PackageError> {
    for character in name.chars() {
        if !is_name_character(character) {
            return Err(PackageError::MalformedName(name.into()));
        }
    }

    Ok(())
}

/// Checks for `Name` safe characters.
fn is_name_character(c: char) -> bool {
    match c {
        '.' | '_' | '-' | '/' => true,
        'a'..='z' => true,
        '0'..='9' => true,
        _ => false,
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A [URL](https://url.spec.whatwg.org/).
///
/// Note that the Frictionless Data spec talks about URIs. This implementation diverges from any
/// ambiguity a URI could impose by using URLs as defined by the WHATWG.
// TODO
pub type Url = String;

/// Represents an [Open Definition licence](https://opendefinition.org/licenses/) under
/// which a package is provided.
///
/// ## Example
///
/// ```
/// use some::package::core::Licence;
///
/// let licence = Licence {
///    name: "ODC-PDDL".into(),
///    path: "http://opendatacommons.org/licenses/pddl/".into(),
///    title: "Open Data Commons Public Domain Dedication and License".into()
/// };
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Licence {
    pub name: String,
    pub path: String,
    pub title: String,
}

impl fmt::Display for Licence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "[{}]({})", &self.name, &self.path)
        } else {
            write!(f, "{}", &self.name)
        }
    }
}

/// Represents an individual or organisation involved in the production or delivery
/// of the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    /// Name of the contributor.
    pub title: String,
    /// Web location of the contributor.
    pub path: Option<Url>,
    pub email: Option<String>,
    /// The organisation the contributor belongs to.
    pub organization: Option<String>,
    /// The role the contributor is playing.
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Author,
    Contributor,
    Maintainer,
    Publisher,
    Wrangler,
}

impl Default for Role {
    fn default() -> Self {
        Role::Contributor
    }
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("Name `{0}` is invalid. A name must only contain lowercase, `.`, `_`, `-`.")]
    MalformedName(String),
    #[error("Profile `{0}` is unknown.")]
    UnknownProfile(String),
    #[error("Field `{0}` is required.")]
    RequiredField(String),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod name {
        use super::*;

        #[test]
        fn assert_valid_name() {
            assert!(assert_name("foo-bar").is_ok(), "Expect name to be valid");
        }
    }
}
