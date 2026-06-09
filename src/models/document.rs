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
    Columns {
        ratios: Vec<f32>,
        gutter: Option<String>,
        children: Vec<StyledColumn>,
    },
    Section {
        style: Option<BlockStyle>,
        children: Vec<DocumentNode>,
    },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StyledColumn {
    pub style: Option<BlockStyle>,
    pub nodes: Vec<DocumentNode>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BlockStyle {
    pub background: Option<String>,
    pub text_color: Option<String>,
    pub border: Option<Border>,
    pub padding: Option<String>,
    pub radius: Option<String>,
}

impl BlockStyle {
    pub fn is_empty(&self) -> bool {
        self.background.is_none()
            && self.text_color.is_none()
            && self.border.is_none()
            && self.padding.is_none()
            && self.radius.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Border {
    pub width: Option<String>,
    pub color: Option<String>,
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
