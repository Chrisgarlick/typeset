use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, CodeBlockKind};

use crate::models::document::{DocumentNode, Frontmatter, ParsedDocument};

pub fn parse(markdown: &str) -> ParsedDocument {
    let (frontmatter, content) = extract_frontmatter(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&content, options);
    let nodes = build_nodes(parser);

    ParsedDocument { frontmatter, nodes }
}

fn extract_frontmatter(input: &str) -> (Option<Frontmatter>, String) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (None, input.to_string());
    }

    // Find the closing ---
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.trim_start_matches(['\r', '\n']);

    if let Some(end_pos) = after_opening.find("\n---") {
        let yaml = &after_opening[..end_pos];
        let content = &after_opening[end_pos + 4..];
        let content = content.trim_start_matches(['\r', '\n']);
        let fm = parse_frontmatter_yaml(yaml);
        (Some(fm), content.to_string())
    } else {
        (None, input.to_string())
    }
}

fn parse_frontmatter_yaml(yaml: &str) -> Frontmatter {
    let mut fm = Frontmatter::default();
    for line in yaml.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let val = parts[1].trim().trim_matches('"').trim_matches('\'').to_string();
        if val.is_empty() {
            continue;
        }
        match key {
            "title" => fm.title = Some(val),
            "eyebrow" => fm.eyebrow = Some(val),
            "reference" => fm.reference = Some(val),
            "cover" => fm.cover = Some(val),
            "subtitle" => fm.subtitle = Some(val),
            "recipient" => fm.recipient = Some(val),
            "date" => fm.date = Some(val),
            "author" => fm.author = Some(val),
            "document_type" => fm.document_type = Some(val),
            "client" => fm.client = Some(val),
            _ => {}
        }
    }
    fm
}

fn build_nodes<'a>(parser: Parser<'a>) -> Vec<DocumentNode> {
    let mut nodes = Vec::new();
    let mut text_buf = String::new();
    let mut in_heading: Option<u8> = None;
    let mut in_paragraph = false;
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut in_list = false;
    let mut list_ordered = false;
    let mut list_start: u64 = 1;
    let mut list_items: Vec<String> = Vec::new();
    let mut in_list_item = false;
    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_current_row: Vec<String> = Vec::new();
    let mut in_table_head = false;
    let mut image_url = String::new();
    let mut image_alt = String::new();
    let mut in_image = false;
    let mut link_stack: Vec<String> = Vec::new();
    let mut image_title = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = Some(level as u8);
                text_buf.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = in_heading {
                    let text = text_buf.trim().to_string();
                    let id = slugify(&text);
                    nodes.push(DocumentNode::Heading { level, text, id });
                }
                in_heading = None;
                text_buf.clear();
            }

            Event::Start(Tag::Paragraph) => {
                if in_blockquote || in_list_item {
                    // Don't nest — just accumulate text
                } else {
                    in_paragraph = true;
                    text_buf.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if in_blockquote || in_list_item {
                    // handled by parent
                } else if in_paragraph {
                    let text = text_buf.trim().to_string();
                    if !text.is_empty() {
                        // Check for page break marker
                        if text == "{{pagebreak}}" || text == "{{page-break}}" {
                            nodes.push(DocumentNode::PageBreak);
                        } else {
                            nodes.push(DocumentNode::Paragraph { text });
                        }
                    }
                    in_paragraph = false;
                    text_buf.clear();
                }
            }

            Event::Start(Tag::BlockQuote(_)) => {
                in_blockquote = true;
                text_buf.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                let text = text_buf.trim().to_string();
                if !text.is_empty() {
                    nodes.push(DocumentNode::Blockquote { text });
                }
                in_blockquote = false;
                text_buf.clear();
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.to_string();
                        if l.is_empty() { None } else { Some(l) }
                    }
                    CodeBlockKind::Indented => None,
                };
                text_buf.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                let code = text_buf.trim_end().to_string();
                nodes.push(DocumentNode::CodeBlock {
                    language: code_lang.take(),
                    code,
                });
                in_code_block = false;
                text_buf.clear();
            }

            Event::Start(Tag::List(start)) => {
                in_list = true;
                list_items.clear();
                if let Some(s) = start {
                    list_ordered = true;
                    list_start = s;
                } else {
                    list_ordered = false;
                    list_start = 1;
                }
            }
            Event::End(TagEnd::List(_)) => {
                if list_ordered {
                    nodes.push(DocumentNode::OrderedList {
                        items: list_items.clone(),
                        start: list_start,
                    });
                } else {
                    nodes.push(DocumentNode::BulletList {
                        items: list_items.clone(),
                    });
                }
                in_list = false;
                list_items.clear();
            }

            Event::Start(Tag::Item) => {
                in_list_item = true;
                text_buf.clear();
            }
            Event::End(TagEnd::Item) => {
                list_items.push(text_buf.trim().to_string());
                in_list_item = false;
                text_buf.clear();
            }

            Event::Start(Tag::Table(_alignments)) => {
                in_table = true;
                table_headers.clear();
                table_rows.clear();
            }
            Event::End(TagEnd::Table) => {
                nodes.push(DocumentNode::Table {
                    headers: table_headers.clone(),
                    rows: table_rows.clone(),
                });
                in_table = false;
                table_headers.clear();
                table_rows.clear();
            }

            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                table_current_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                table_headers = table_current_row.clone();
                in_table_head = false;
                table_current_row.clear();
            }

            Event::Start(Tag::TableRow) => {
                table_current_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if !in_table_head {
                    table_rows.push(table_current_row.clone());
                }
                table_current_row.clear();
            }

            Event::Start(Tag::TableCell) => {
                text_buf.clear();
            }
            Event::End(TagEnd::TableCell) => {
                table_current_row.push(text_buf.trim().to_string());
                text_buf.clear();
            }

            Event::Start(Tag::Image { dest_url, title, .. }) => {
                in_image = true;
                image_url = dest_url.to_string();
                image_alt.clear();
                image_title = title.to_string();
            }
            Event::End(TagEnd::Image) => {
                // The markdown title (`![alt](url "title")`) is the caption.
                nodes.push(DocumentNode::Image {
                    url: image_url.clone(),
                    alt: image_alt.trim().to_string(),
                    caption: Some(image_title.clone()).filter(|t| !t.is_empty()),
                    full_width: false,
                });
                in_image = false;
                image_url.clear();
                image_alt.clear();
            }

            Event::Text(text) => {
                if in_image {
                    image_alt.push_str(&text);
                } else if in_code_block {
                    text_buf.push_str(&text);
                } else {
                    text_buf.push_str(&escape_inline(&text));
                }
            }

            Event::Code(code) => {
                let fence = if code.contains('`') { "`` " } else { "`" };
                text_buf.push_str(fence);
                text_buf.push_str(&code);
                text_buf.push_str(&fence.chars().rev().collect::<String>());
            }

            // Inline formatting is kept as markdown in the node's text, which
            // the renderers read back (see `typst_inline`).
            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) if !in_image => {
                text_buf.push_str("**");
            }
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) if !in_image => {
                text_buf.push('*');
            }
            Event::Start(Tag::Strikethrough) | Event::End(TagEnd::Strikethrough) if !in_image => {
                text_buf.push_str("~~");
            }
            Event::Start(Tag::Link { dest_url, .. }) if !in_image => {
                text_buf.push('[');
                link_stack.push(dest_url.to_string());
            }
            Event::End(TagEnd::Link) if !in_image => {
                let url = link_stack.pop().unwrap_or_default();
                text_buf.push_str("](");
                text_buf.push_str(&url.replace(' ', "%20").replace(')', "%29"));
                text_buf.push(')');
            }

            Event::SoftBreak => {
                text_buf.push(' ');
            }

            Event::HardBreak => {
                text_buf.push('\n');
            }

            Event::Rule => {
                nodes.push(DocumentNode::HorizontalRule);
            }

            _ => {}
        }
    }

    nodes
}

/// Backslash-escape the characters inline markdown gives meaning to, so
/// literal text survives being parsed again as inline markdown.
fn escape_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '`' | '[' | ']' | '~' | '<' | '&') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_formatting_survives_parsing() {
        let doc = parse("A **bold** and *soft* [link](https://example.com), with `x` and 5 * 3.");
        let DocumentNode::Paragraph { text } = &doc.nodes[0] else {
            panic!("expected a paragraph");
        };
        assert_eq!(text, r"A **bold** and *soft* [link](https://example.com), with `x` and 5 \* 3.");
    }

    #[test]
    fn code_blocks_are_not_escaped() {
        let doc = parse("```\nlet a = b * c_d;\n```");
        let DocumentNode::CodeBlock { code, .. } = &doc.nodes[0] else {
            panic!("expected a code block");
        };
        assert_eq!(code, "let a = b * c_d;");
    }

    #[test]
    fn test_parse_frontmatter() {
        let md = r#"---
title: My Document
subtitle: A test
author: Chris
---

# Hello World

Some content here."#;

        let doc = parse(md);
        let fm = doc.frontmatter.unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Document"));
        assert_eq!(fm.subtitle.as_deref(), Some("A test"));
        assert_eq!(fm.author.as_deref(), Some("Chris"));
        assert!(doc.nodes.len() >= 2);
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let md = "# Hello\n\nWorld";
        let doc = parse(md);
        assert!(doc.frontmatter.is_none());
        assert!(!doc.nodes.is_empty());
    }

    #[test]
    fn test_heading_levels() {
        let md = "# H1\n\n## H2\n\n### H3";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::Heading { level, text, .. } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "H1");
            }
            _ => panic!("Expected heading"),
        }
    }

    #[test]
    fn test_bullet_list() {
        let md = "- Item one\n- Item two\n- Item three";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::BulletList { items } => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "Item one");
            }
            _ => panic!("Expected bullet list"),
        }
    }

    #[test]
    fn test_ordered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::OrderedList { items, start } => {
                assert_eq!(items.len(), 3);
                assert_eq!(*start, 1);
            }
            _ => panic!("Expected ordered list"),
        }
    }

    #[test]
    fn test_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::CodeBlock { language, code } => {
                assert_eq!(language.as_deref(), Some("rust"));
                assert_eq!(code, "fn main() {}");
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn test_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::Table { headers, rows } => {
                assert_eq!(headers, &["A", "B"]);
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0], &["1", "2"]);
            }
            _ => panic!("Expected table"),
        }
    }

    #[test]
    fn test_blockquote() {
        let md = "> This is a quote";
        let doc = parse(md);
        match &doc.nodes[0] {
            DocumentNode::Blockquote { text } => {
                assert_eq!(text, "This is a quote");
            }
            _ => panic!("Expected blockquote"),
        }
    }

    #[test]
    fn test_horizontal_rule() {
        let md = "---";
        let doc = parse(md);
        assert!(matches!(doc.nodes[0], DocumentNode::HorizontalRule));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("API Reference (v2)"), "api-reference-v2");
    }
}
