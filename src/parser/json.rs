use serde::Deserialize;

use crate::models::components::Component;
use crate::models::document::{
    BlockStyle, Border, DocumentNode, Frontmatter, ParsedDocument, StyledColumn,
};

pub fn parse(input: &str) -> anyhow::Result<ParsedDocument> {
    let doc: JsonDocument = serde_json::from_str(input)
        .map_err(|e| anyhow::anyhow!("invalid JSON document: {e}"))?;
    let schema = doc.schema.unwrap_or(1);
    if !(1..=2).contains(&schema) {
        anyhow::bail!("invalid JSON document: `schema` must be 1 or 2, not {schema}");
    }

    let mut nodes = Vec::new();
    for (idx, page) in doc.pages.iter().enumerate() {
        if idx > 0 {
            nodes.push(DocumentNode::SectionBreak);
        }
        for (b, value) in page.blocks.iter().enumerate() {
            let path = format!("pages[{idx}].blocks[{b}]");
            push_value(&mut nodes, value, schema, &path)?;
        }
    }

    Ok(ParsedDocument {
        frontmatter: doc.frontmatter.map(Into::into),
        nodes,
    })
}

/// Parse one block. Components (schema 2) are checked strictly and their
/// errors name the block's path; everything else is a v1 block.
fn push_value(out: &mut Vec<DocumentNode>, value: &serde_json::Value, schema: u32, path: &str) -> anyhow::Result<()> {
    let kind = value.get("type").and_then(|t| t.as_str()).unwrap_or_default();
    if Component::TYPES.contains(&kind) {
        if schema < 2 {
            anyhow::bail!("{path}: the `{kind}` block needs \"schema\": 2 at the top of the document");
        }
        let mut fields = value.clone();
        if let Some(map) = fields.as_object_mut() {
            map.remove("type");
        }
        let component = Component::from_json(kind, fields).map_err(|e| anyhow::anyhow!("{path} ({kind}): {e}"))?;
        out.push(DocumentNode::Component(component));
        return Ok(());
    }
    let block: Block = serde_json::from_value(value.clone()).map_err(|e| anyhow::anyhow!("{path}: {e}"))?;
    push_block(out, &block, schema, path)
}

fn push_block(out: &mut Vec<DocumentNode>, block: &Block, schema: u32, path: &str) -> anyhow::Result<()> {
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
        Block::Image { url, alt, caption, width } => {
            if schema >= 2 && alt.as_deref().is_none_or(|a| a.trim().is_empty()) {
                anyhow::bail!("{path} (image): needs `alt` text describing the image, for screen readers");
            }
            out.push(DocumentNode::Image {
                url: url.clone(),
                alt: alt.clone().unwrap_or_default(),
                caption: caption.clone(),
                full_width: width.as_deref() == Some("full"),
            });
        }
        Block::Rule => out.push(DocumentNode::HorizontalRule),
        Block::PageBreak => out.push(DocumentNode::PageBreak),
        Block::Columns { ratios, gutter, children } => {
            let mut columns: Vec<StyledColumn> = Vec::new();
            for (c, slot) in children.iter().enumerate() {
                let mut nodes = Vec::new();
                for (b, value) in slot.blocks.iter().enumerate() {
                    push_value(&mut nodes, value, schema, &format!("{path}.children[{c}].blocks[{b}]"))?;
                }
                columns.push(StyledColumn {
                    style: slot.style().filter(|s| !s.is_empty()),
                    nodes,
                });
            }
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
            for (b, value) in blocks.iter().enumerate() {
                push_value(&mut children, value, schema, &format!("{path}.blocks[{b}]"))?;
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
    Ok(())
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
    /// 1 (the default) or 2, which enables components.
    #[serde(default)]
    schema: Option<u32>,
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
    #[serde(default)]
    eyebrow: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    cover: Option<String>,
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
            eyebrow: f.eyebrow,
            reference: f.reference,
            cover: f.cover,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Page {
    #[serde(default)]
    blocks: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Slot {
    #[serde(default)]
    blocks: Vec<serde_json::Value>,
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
        #[serde(default)]
        caption: Option<String>,
        /// `column` (the default) or `full`.
        #[serde(default)]
        width: Option<String>,
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
        blocks: Vec<serde_json::Value>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_need_schema_2() {
        let input = r#"{ "pages": [{ "blocks": [{ "type": "facts", "items": [] }] }] }"#;
        let err = parse(input).unwrap_err().to_string();
        assert!(err.contains("pages[0].blocks[0]"), "{err}");
        assert!(err.contains("\"schema\": 2"), "{err}");
    }

    #[test]
    fn component_errors_name_the_block_and_field() {
        let input = r#"{ "schema": 2, "pages": [{ "blocks": [
            { "type": "paragraph", "text": "fine" },
            { "type": "findings", "items": [{ "title": "x", "severity": "high" }] }
        ]}]}"#;
        let err = parse(input).unwrap_err().to_string();
        assert!(err.contains("pages[0].blocks[1] (findings)"), "{err}");
        assert!(err.contains("high"), "{err}");
    }

    #[test]
    fn components_reject_unknown_fields() {
        let input = r#"{ "schema": 2, "pages": [{ "blocks": [
            { "type": "callout", "body": "x", "colour": "red" }
        ]}]}"#;
        let err = parse(input).unwrap_err().to_string();
        assert!(err.contains("unknown field `colour`"), "{err}");
    }

    #[test]
    fn components_parse_inside_layout_blocks() {
        let input = r#"{ "schema": 2, "pages": [{ "blocks": [
            { "type": "section", "blocks": [{ "type": "callout", "tone": "warn", "body": "x" }] }
        ]}]}"#;
        let doc = parse(input).unwrap();
        let DocumentNode::Section { children, .. } = &doc.nodes[0] else { panic!("expected a section") };
        assert!(matches!(children[0], DocumentNode::Component(Component::Callout(_))));
    }

    #[test]
    fn highlights_take_one_to_four_items() {
        let item = r#"{ "title": "x", "severity": "minor" }"#;
        let doc = |n: usize| {
            format!(
                r#"{{ "schema": 2, "pages": [{{ "blocks": [{{ "type": "highlights", "items": [{}] }}] }}] }}"#,
                vec![item; n].join(",")
            )
        };
        assert!(parse(&doc(4)).is_ok());
        assert!(parse(&doc(5)).unwrap_err().to_string().contains("one to four"));
        assert!(parse(&doc(0)).is_err());
    }

    #[test]
    fn images_carry_caption_and_width() {
        let doc = parse(r#"{ "pages": [{ "blocks": [
            { "type": "image", "url": "https://example.com/a.png", "alt": "A", "caption": "Cap", "width": "full" }
        ]}]}"#).unwrap();
        let DocumentNode::Image { caption, full_width, .. } = &doc.nodes[0] else { panic!("expected an image") };
        assert_eq!(caption.as_deref(), Some("Cap"));
        assert!(*full_width);
    }

    #[test]
    fn schema_2_images_need_alt_text() {
        let doc = |schema: u32| format!(r#"{{ "schema": {schema}, "pages": [{{ "blocks": [{{ "type": "image", "url": "https://a.test/x.png" }}] }}] }}"#);
        assert!(parse(&doc(1)).is_ok(), "version 1 documents render as before");
        assert!(parse(&doc(2)).unwrap_err().to_string().contains("needs `alt` text"));
    }

    #[test]
    fn rejects_unknown_schema_versions() {
        assert!(parse(r#"{ "schema": 3, "pages": [] }"#).is_err());
    }

    #[test]
    fn the_gallery_parses() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/gallery.json");
        parse(&std::fs::read_to_string(path).unwrap()).expect("gallery.json");
    }

    #[test]
    fn the_json_schema_covers_every_component() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/schema/v2.json");
        let schema: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for kind in Component::TYPES {
            assert_eq!(schema["$defs"][kind]["properties"]["type"]["const"], kind, "schema/v2.json is missing `{kind}`");
        }
    }

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
    fn inserts_section_break_between_pages() {
        let input = r#"{
            "pages": [
                { "blocks": [ { "type": "paragraph", "text": "a" } ] },
                { "blocks": [ { "type": "paragraph", "text": "b" } ] }
            ]
        }"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(doc.nodes[1], DocumentNode::SectionBreak));
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
