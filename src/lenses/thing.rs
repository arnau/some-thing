use scraper::{Html, Selector};
use url::Url;

use crate::package::core::Name;
use crate::package::resource::{
    Constraint, Field, ForeignKey, Reference, Resource, ResourceBuilder, Schema,
};
use crate::store::{Store, ThingStore};
use crate::{thing::NewThing, Report, Result};

pub fn add(store: &mut Store, thing: NewThing) -> Result<Report> {
    ThingStore::write(store, thing)?;

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

/// The thing resouce for the package.
pub fn package_resource() -> Resource {
    let schema = Schema {
        fields: vec![
            Field {
                name: Name::new("url"),
                description: "The URL of the thing.".into(),
                datatype: "string".into(),
                format: Some("uri".into()),
                constraints: vec![Constraint {
                    required: true,
                    unique: true,
                }],
            },
            Field {
                name: Name::new("name"),
                description: "The name of the thing.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: true,
                    unique: true,
                }],
            },
            Field {
                name: Name::new("summary"),
                description: "The description of the thing.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: false,
                    unique: false,
                }],
            },
            Field {
                name: Name::new("category_id"),
                description: "The category of the thing.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: true,
                    unique: false,
                }],
            },
        ],

        primary_key: vec![Name::new("url")],
        foreign_keys: vec![ForeignKey {
            fields: vec![Name::new("category_id")],
            reference: Reference {
                resource: Name::new("tag"),
                fields: vec![Name::new("id")],
            },
        }],
    };

    let mut builder = ResourceBuilder::new();
    builder.with_name(Name::new("thing"));
    builder.with_title("Thing");
    builder.with_description("The set of things for the collection.");
    builder.with_path("data/thing.csv");
    builder.with_schema(schema);

    let resource = builder.build();

    resource
}
