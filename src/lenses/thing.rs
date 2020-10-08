use crate::store::Store;
use crate::{Report, Result};

pub fn add(store: &Store) -> Result<Report> {
    Ok(Report::new("Success"))
}
