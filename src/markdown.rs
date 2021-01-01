use std::fmt;

/// `Markdown` is similar to [`Display`], but expects to output valid Markdown.
///
/// No guarantees enforced.
pub trait Markdown {
    fn fmt_md(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>;
}
