use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum DocumentNode {
    Heading {
        level: u8,
        text: String,
        id: String,
    },
    Paragraph {
        text: String,
    },
    BulletList {
        items: Vec<String>,
    },
    OrderedList {
        items: Vec<String>,
        start: u64,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Blockquote {
        text: String,
    },
    HorizontalRule,
    Image {
        url: String,
        alt: String,
    },
    PageBreak,
}

#[derive(Debug, Serialize)]
pub struct ParsedDocument {
    pub frontmatter: Option<Frontmatter>,
    pub nodes: Vec<DocumentNode>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub recipient: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
    pub document_type: Option<String>,
    pub client: Option<String>,
}
