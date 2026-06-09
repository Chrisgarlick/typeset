use serde::Deserialize;

use crate::models::document::{
    BlockStyle, Border, DocumentNode, Frontmatter, ParsedDocument, StyledColumn,
};

pub fn parse(input: &str) -> anyhow::Result<ParsedDocument> {
    let doc: JsonDocument = serde_json::from_str(input)
        .map_err(|e| anyhow::anyhow!("invalid JSON document: {e}"))?;

    let mut nodes = Vec::new();
    for (idx, page) in doc.pages.iter().enumerate() {
        if idx > 0 {
            nodes.push(DocumentNode::PageBreak);
        }
        for block in &page.blocks {
            push_block(&mut nodes, block);
        }
    }

    Ok(ParsedDocument {
        frontmatter: doc.frontmatter.map(Into::into),
        nodes,
    })
}

fn push_block(out: &mut Vec<DocumentNode>, block: &Block) {
    match block {
        Block::Heading { level, text } => {
            let id = slugify(text);
            out.push(DocumentNode::Heading {
                level: (*level).clamp(1, 6),
                text: text.clone(),
                id,
            });
        }
        Block::Paragraph { text } => {
            out.push(DocumentNode::Paragraph { text: text.clone() });
        }
        Block::Markdown { content } => {
            let parsed = crate::parser::markdown::parse(content);
            out.extend(parsed.nodes);
        }
        Block::BulletList { items } => {
            out.push(DocumentNode::BulletList { items: items.clone() });
        }
        Block::OrderedList { items, start } => {
            out.push(DocumentNode::OrderedList {
                items: items.clone(),
                start: start.unwrap_or(1),
            });
        }
        Block::Table { headers, rows } => {
            out.push(DocumentNode::Table {
                headers: headers.clone(),
                rows: rows.clone(),
            });
        }
        Block::Code { language, content } => {
            out.push(DocumentNode::CodeBlock {
                language: language.clone(),
                code: content.clone(),
            });
        }
        Block::Blockquote { text } => {
            out.push(DocumentNode::Blockquote { text: text.clone() });
        }
        Block::Image { url, alt } => {
            out.push(DocumentNode::Image {
                url: url.clone(),
                alt: alt.clone().unwrap_or_default(),
            });
        }
        Block::Rule => out.push(DocumentNode::HorizontalRule),
        Block::PageBreak => out.push(DocumentNode::PageBreak),
        Block::Columns { ratios, gutter, children } => {
            let columns: Vec<StyledColumn> = children
                .iter()
                .map(|slot| {
                    let mut nodes = Vec::new();
                    for b in &slot.blocks {
                        push_block(&mut nodes, b);
                    }
                    StyledColumn {
                        style: slot.style().filter(|s| !s.is_empty()),
                        nodes,
                    }
                })
                .collect();
            out.push(DocumentNode::Columns {
                ratios: ratios.clone(),
                gutter: gutter.clone(),
                children: columns,
            });
        }
        Block::Section {
            background,
            text_color,
            border,
            padding,
            radius,
            blocks,
        } => {
            let mut children = Vec::new();
            for b in blocks {
                push_block(&mut children, b);
            }
            let style = BlockStyle {
                background: background.clone(),
                text_color: text_color.clone(),
                border: border.clone().map(Into::into),
                padding: padding.clone(),
                radius: radius.clone(),
            };
            out.push(DocumentNode::Section {
                style: if style.is_empty() { None } else { Some(style) },
                children,
            });
        }
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

// ---------------------------------------------------------------------------
// Schema mirror
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JsonDocument {
    #[serde(default)]
    frontmatter: Option<JsonFrontmatter>,
    pages: Vec<Page>,
}

#[derive(Debug, Deserialize, Default)]
struct JsonFrontmatter {
    title: Option<String>,
    subtitle: Option<String>,
    recipient: Option<String>,
    date: Option<String>,
    author: Option<String>,
    document_type: Option<String>,
    client: Option<String>,
}

impl From<JsonFrontmatter> for Frontmatter {
    fn from(f: JsonFrontmatter) -> Self {
        Frontmatter {
            title: f.title,
            subtitle: f.subtitle,
            recipient: f.recipient,
            date: f.date,
            author: f.author,
            document_type: f.document_type,
            client: f.client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Page {
    #[serde(default)]
    blocks: Vec<Block>,
}

#[derive(Debug, Deserialize)]
struct Slot {
    #[serde(default)]
    blocks: Vec<Block>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    text_color: Option<String>,
    #[serde(default)]
    border: Option<JsonBorder>,
    #[serde(default)]
    padding: Option<String>,
    #[serde(default)]
    radius: Option<String>,
}

impl Slot {
    fn style(&self) -> Option<BlockStyle> {
        let s = BlockStyle {
            background: self.background.clone(),
            text_color: self.text_color.clone(),
            border: self.border.clone().map(Into::into),
            padding: self.padding.clone(),
            radius: self.radius.clone(),
        };
        Some(s)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct JsonBorder {
    #[serde(default)]
    width: Option<String>,
    #[serde(default)]
    color: Option<String>,
}

impl From<JsonBorder> for Border {
    fn from(b: JsonBorder) -> Self {
        Border {
            width: b.width,
            color: b.color,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    Markdown {
        content: String,
    },
    BulletList {
        items: Vec<String>,
    },
    OrderedList {
        items: Vec<String>,
        #[serde(default)]
        start: Option<u64>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Code {
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    Blockquote {
        text: String,
    },
    Image {
        url: String,
        #[serde(default)]
        alt: Option<String>,
    },
    Rule,
    PageBreak,
    Columns {
        ratios: Vec<f32>,
        #[serde(default)]
        gutter: Option<String>,
        children: Vec<Slot>,
    },
    Section {
        #[serde(default)]
        background: Option<String>,
        #[serde(default)]
        text_color: Option<String>,
        #[serde(default)]
        border: Option<JsonBorder>,
        #[serde(default)]
        padding: Option<String>,
        #[serde(default)]
        radius: Option<String>,
        #[serde(default)]
        blocks: Vec<Block>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_doc() {
        let input = r#"{
            "frontmatter": { "title": "Hello" },
            "pages": [
                { "blocks": [ { "type": "heading", "level": 2, "text": "Intro" } ] }
            ]
        }"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.frontmatter.as_ref().unwrap().title.as_deref(), Some("Hello"));
        assert_eq!(doc.nodes.len(), 1);
        assert!(matches!(doc.nodes[0], DocumentNode::Heading { level: 2, .. }));
    }

    #[test]
    fn inserts_pagebreak_between_pages() {
        let input = r#"{
            "pages": [
                { "blocks": [ { "type": "paragraph", "text": "a" } ] },
                { "blocks": [ { "type": "paragraph", "text": "b" } ] }
            ]
        }"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(doc.nodes[1], DocumentNode::PageBreak));
    }

    #[test]
    fn columns_with_markdown_slots() {
        let input = r#"{
            "pages": [{
                "blocks": [{
                    "type": "columns",
                    "ratios": [1, 1],
                    "children": [
                        { "blocks": [ { "type": "markdown", "content": "**Wins**\n\n- A\n- B" } ] },
                        { "blocks": [ { "type": "markdown", "content": "Risks paragraph." } ] }
                    ]
                }]
            }]
        }"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        match &doc.nodes[0] {
            DocumentNode::Columns { ratios, children, .. } => {
                assert_eq!(ratios, &[1.0, 1.0]);
                assert_eq!(children.len(), 2);
                // First slot should have a paragraph + bullet list from markdown
                assert!(children[0].nodes.len() >= 2);
                // Second slot has a single paragraph
                assert!(matches!(children[1].nodes[0], DocumentNode::Paragraph { .. }));
                // No styling specified — style should be None
                assert!(children[0].style.is_none());
            }
            _ => panic!("expected columns"),
        }
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse("not json").is_err());
    }

    #[test]
    fn columns_carry_per_slot_styling() {
        let input = r##"{
            "pages": [{
                "blocks": [{
                    "type": "columns",
                    "ratios": [1, 1],
                    "children": [
                        {
                            "background": "#1A202C",
                            "text_color": "#FFFFFF",
                            "padding": "14pt",
                            "blocks": [ { "type": "paragraph", "text": "dark" } ]
                        },
                        {
                            "background": "#FFFFFF",
                            "border": { "width": "1pt", "color": "#E5E7EB" },
                            "padding": "14pt",
                            "blocks": [ { "type": "paragraph", "text": "light" } ]
                        }
                    ]
                }]
            }]
        }"##;
        let doc = parse(input).unwrap();
        match &doc.nodes[0] {
            DocumentNode::Columns { children, .. } => {
                let left = children[0].style.as_ref().expect("left styled");
                assert_eq!(left.background.as_deref(), Some("#1A202C"));
                assert_eq!(left.text_color.as_deref(), Some("#FFFFFF"));
                let right = children[1].style.as_ref().expect("right styled");
                assert_eq!(right.background.as_deref(), Some("#FFFFFF"));
                let border = right.border.as_ref().expect("right border");
                assert_eq!(border.width.as_deref(), Some("1pt"));
                assert_eq!(border.color.as_deref(), Some("#E5E7EB"));
            }
            _ => panic!("expected columns"),
        }
    }
}
