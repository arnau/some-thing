use scraper::{Html, Selector};
use url::Url;

use crate::store::Store;
use crate::{Report, Result};

pub fn add(store: &Store) -> Result<Report> {
    Ok(Report::new("Success"))
}

pub fn validate_url<S: Into<String>>(input: S) -> Result<()> {
    Url::parse(&input.into())?;

    Ok(())
}

pub fn fetch_thing(url: &str) -> Result<()> {
    let url = Url::parse(url)?;
    let req = reqwest::blocking::get(url)?;

    let status = req.status();
    let body = req.text()?;

    dbg!(status);

    let document = Html::parse_document(&body);
    let title_selector = Selector::parse("title").expect("valid css selector");
    let desc_selector = Selector::parse(r#"meta[name="description"]"#).expect("valid css selector");

    for element in document.select(&title_selector) {
        dbg!(&element.text().collect::<String>());
    }

    for element in document.select(&desc_selector) {
        dbg!(&element.value().attr("content"));
    }

    Ok(())
}
