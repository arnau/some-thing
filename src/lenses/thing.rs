use scraper::{Html, Selector};
use url::Url;

use crate::{Result, SomeError};
use crate::info;


pub fn validate_url<S: Into<String>>(input: S) -> Result<()> {
    Url::parse(&input.into())?;

    Ok(())
}

pub fn fetch_thing(input: &str) -> Result<()> {
    info!("Fetching information about:", input);

    let url = Url::parse(input)?;
    let req = reqwest::blocking::get(url)?;
    let status = req.status();
    let body = req.text()?;

    if status != 200 {
        return Err(SomeError::BadUrl(input.to_string()));
    }

    let document = Html::parse_document(&body);
    let title_selector = Selector::parse("title").expect("valid css selector");
    let desc_selector = Selector::parse(r#"meta[name="description"]"#).expect("valid css selector");

    for element in document.select(&title_selector) {
        info!("Found a title:", &element.text().collect::<String>());
    }

    for element in document.select(&desc_selector) {
        if let Some(summary) = &element.value().attr("content") {
            info!("Found a summary:", summary);
        }
    }

    Ok(())
}
