use crate::package::core::Name;
use crate::package::resource::{
    Constraint, Field, ForeignKey, Reference, Resource, ResourceBuilder, Schema,
};

/// Builds the resources for a Some package.
pub fn resources() -> Vec<Resource> {
    vec![thing_resource(), tag_resource(), thing_tag_resource()]
}

pub fn tag_resource() -> Resource {
    let schema = Schema {
        fields: vec![
            Field {
                name: Name::new("id"),
                description: "The tag identifier.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: true,
                    unique: true,
                }],
            },
            Field {
                name: Name::new("name"),
                description: "The tag human readable name.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: true,
                    unique: true,
                }],
            },
            Field {
                name: Name::new("summary"),
                description: "The tag description.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: false,
                    unique: false,
                }],
            },
        ],
        primary_key: vec![Name::new("id")],
        foreign_keys: vec![],
    };

    let mut builder = ResourceBuilder::new();
    builder.with_name(Name::new("tag"));
    builder.with_title("Tag");
    builder.with_description("The set of tags to classify the collection of things.");
    builder.with_path("data/tag.csv");
    builder.with_schema(schema);

    let resource = builder.build();

    resource
}

/// The thing resouce.
pub fn thing_resource() -> Resource {
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

/// The thing_tag resouce.
pub fn thing_tag_resource() -> Resource {
    let schema = Schema {
        fields: vec![
            Field {
                name: Name::new("thing_id"),
                description: "The reference to a thing.".into(),
                datatype: "string".into(),
                format: Some("uri".into()),
                constraints: vec![Constraint {
                    required: true,
                    unique: false,
                }],
            },
            Field {
                name: Name::new("tag_id"),
                description: "The reference to a tag.".into(),
                datatype: "string".into(),
                format: None,
                constraints: vec![Constraint {
                    required: true,
                    unique: false,
                }],
            },
        ],
        primary_key: vec![Name::new("thing_id"), Name::new("tag_id")],
        foreign_keys: vec![
            ForeignKey {
                fields: vec![Name::new("thing_id")],
                reference: Reference {
                    resource: Name::new("thing"),
                    fields: vec![Name::new("url")],
                },
            },
            ForeignKey {
                fields: vec![Name::new("tag_id")],
                reference: Reference {
                    resource: Name::new("tag"),
                    fields: vec![Name::new("id")],
                },
            },
        ],
    };

    let mut builder = ResourceBuilder::new();
    builder.with_name(Name::new("thing_tag"));
    builder.with_title("Thing tags");
    builder.with_description("The set of tags to further classify the collection of things.");
    builder.with_path("data/thing_tag.csv");
    builder.with_schema(schema);

    let resource = builder.build();

    resource
}
