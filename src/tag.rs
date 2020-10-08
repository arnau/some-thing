use std::fmt;

pub type Icon = Vec<u8>;
pub type TagId = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    id: TagId,
    name: Option<String>,
    summary: Option<String>,
    icon: Option<Icon>,
}

impl Tag {
    pub fn new(
        id: TagId,
        name: Option<String>,
        summary: Option<String>,
        icon: Option<Icon>,
    ) -> Self {
        Self {
            id,
            name,
            summary,
            icon,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug)]
pub struct TagSet(Vec<Tag>);

impl TagSet {
    pub fn new(raw: Vec<Tag>) -> Self {
        Self(raw)
    }

    pub fn as_slice(&self) -> &[Tag] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Tag> {
        self.0.clone()
    }
}

impl fmt::Display for TagSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list: Vec<String> = self.0.iter().map(|tag| format!("{}", tag)).collect();

        if f.alternate() {
            write!(f, "{}", list.join("\n"))
        } else {
            write!(f, "{}", list.join(";"))
        }
    }
}
