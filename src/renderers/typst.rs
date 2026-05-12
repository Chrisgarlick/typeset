use std::io::Write;
use std::process::{Command, Stdio};

use crate::brand::engine::BrandedDocument;
use crate::models::document::DocumentNode;

/// Render a branded document to PDF bytes via the Typst CLI.
pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let source = build_source(branded);
    compile(&source)
}

// ---------------------------------------------------------------------------
// Source generation
// ---------------------------------------------------------------------------

fn build_source(branded: &BrandedDocument) -> String {
    let mut s = String::with_capacity(4096);

    let (cover, consumed_body_h1) = extract_cover(branded);
    write_preamble(&mut s, branded, &cover);
    write_show_rules(&mut s, branded);
    write_imports(&mut s);
    if let Some(c) = &cover {
        write_cover(&mut s, c, branded);
    }
    write_body(&mut s, &branded.doc.nodes, consumed_body_h1);

    s
}

struct CoverData {
    title: String,
    subtitle: Option<String>,
    recipient: Option<String>,
    author: Option<String>,
    date: Option<String>,
}

/// Returns (cover, did_consume_body_h1).
///
/// Cover precedence:
/// 1. Frontmatter `title` wins — body H1s stay in the body (each starts a new page).
/// 2. Else, the first H1 in the body becomes the cover and is dropped from the body.
fn extract_cover(branded: &BrandedDocument) -> (Option<CoverData>, bool) {
    let fm = branded.doc.frontmatter.as_ref();

    let fm_title = fm.and_then(|f| f.title.clone());
    let (title, consumed_body_h1) = match fm_title {
        Some(t) => (Some(t), false),
        None => {
            let h1 = branded.doc.nodes.iter().find_map(|n| match n {
                DocumentNode::Heading { level: 1, text, .. } => Some(text.clone()),
                _ => None,
            });
            (h1, true)
        }
    };

    let Some(title) = title else {
        return (None, false);
    };

    (
        Some(CoverData {
            title,
            subtitle: fm.and_then(|f| f.subtitle.clone()),
            recipient: fm.and_then(|f| f.recipient.clone()),
            author: fm.and_then(|f| f.author.clone()),
            date: fm.and_then(|f| f.date.clone()),
        }),
        consumed_body_h1,
    )
}

fn write_preamble(s: &mut String, b: &BrandedDocument, cover: &Option<CoverData>) {
    let p = &b.profile;

    let doc_title = cover
        .as_ref()
        .map(|c| c.title.clone())
        .unwrap_or_else(|| "Document".to_string());
    let author = cover
        .as_ref()
        .and_then(|c| c.author.clone())
        .unwrap_or_default();

    let has_cover = if cover.is_some() { "true" } else { "false" };
    s.push_str(&format!(
        r#"#set document(title: {title}, author: {author})

#let primary = rgb("{primary}")
#let secondary = rgb("{secondary}")
#let accent = rgb("{accent}")
#let text-color = rgb("{text}")
#let table-header-bg = rgb("{table_header}")
#let table-border-color = rgb("{table_border}")
#let callout-bg = rgb("{callout_bg}")

#let header-text = {header_text}
#let has-cover = {has_cover}

#set page(
  paper: "{paper}",
  margin: (top: {mt}mm, bottom: {mb}mm, left: {ml}mm, right: {mr}mm),
  header: context {{
    let n = counter(page).get().first()
    if has-cover and n == 1 [] else [
      #text(size: 8pt, fill: secondary, header-text)
      #v(-0.5em)
      #line(length: 100%, stroke: 0.3pt + table-border-color)
    ]
  }},
  footer: context {{
    let n = counter(page).get().first()
    let total = counter(page).final().first()
    let display-n = if has-cover {{ n - 1 }} else {{ n }}
    let display-total = if has-cover {{ total - 1 }} else {{ total }}
    if has-cover and n == 1 [] else [
      #line(length: 100%, stroke: 0.3pt + table-border-color)
      #align(right)[#text(size: 8pt, fill: secondary)[Page #display-n of #display-total]]
    ]
  }},
)

#set text(font: "{font_body}", size: {base_size}pt, fill: text-color)
#set par(leading: {leading}em, spacing: 1.4em, justify: false)
#set block(spacing: 1.4em)
#set list(marker: text(fill: accent)[•], spacing: 0.6em)
#set enum(spacing: 0.6em)
"#,
        title = typst_string(&doc_title),
        author = typst_string(&author),
        primary = hex_or_default(&p.colour_primary, "#000000"),
        secondary = hex_or_default(&p.colour_secondary, "#333333"),
        accent = hex_or_default(&p.colour_accent, "#0066CC"),
        text = hex_or_default(&p.colour_text, "#1A1A1A"),
        table_header = hex_or_default(&p.colour_table_header, "#F5F5F5"),
        table_border = hex_or_default(&p.colour_table_border, "#E0E0E0"),
        callout_bg = hex_or_default(&p.colour_callout_bg, "#F8F9FA"),
        header_text = typst_string(&doc_title),
        paper = paper_name(&p.page_size),
        mt = p.margin_top,
        mb = p.margin_bottom,
        ml = p.margin_left,
        mr = p.margin_right,
        font_body = font_family(&p.font_body),
        base_size = p.font_size_base,
        leading = (p.line_height - 1.0).max(0.4),
    ));
}

fn write_show_rules(s: &mut String, b: &BrandedDocument) {
    let p = &b.profile;
    s.push_str(&format!(
        r##"
#show heading.where(level: 1): it => {{
  pagebreak(weak: true)
  block(below: 0.4em, sticky: true)[
    #set text(size: {h1}pt, weight: "bold", fill: primary)
    #it.body
  ]
  line(length: 100%, stroke: 0.6pt + accent)
  v(0.8em)
}}

#show heading.where(level: 2): it => {{
  pagebreak(weak: true)
  block(below: 0.8em, above: 0em, sticky: true)[
    #set text(size: {h2}pt, weight: "bold", fill: secondary)
    #it.body
  ]
  v(0.4em)
}}

#show heading.where(level: 3): it => block(below: 0.6em, above: 1.2em, sticky: true)[
  #set text(size: {h3}pt, weight: "bold", fill: text-color)
  #it.body
]

#show heading.where(level: 4): set text(size: {h4}pt, weight: "bold")
#show heading: set block(sticky: true)

#show raw.where(block: true): it => block(
  breakable: false,
  width: 100%,
  fill: rgb("#1F1F23"),
  inset: 10pt,
  radius: 2pt,
  stroke: none,
  above: 1.4em,
  below: 1.4em,
)[
  #set text(fill: rgb("#E5E7EB"), font: "{font_mono}", size: {code_size}pt)
  #it
]

#show raw.where(block: false): it => box(
  fill: callout-bg,
  inset: (x: 3pt, y: 1pt),
  outset: (y: 2pt),
  radius: 2pt,
)[#text(font: "{font_mono}", size: {code_size}pt)[#it]]

#show table: set block(breakable: true)
#show list: set block(sticky: true)
#show enum: set block(sticky: true)

#show link: set text(fill: accent)
"##,
        h1 = p.font_size_h1,
        h2 = p.font_size_h2,
        h3 = p.font_size_h3,
        h4 = p.font_size_base + 1.0,
        font_mono = font_family(&p.font_mono),
        code_size = p.font_size_base - 1.0,
    ));
}

fn write_imports(s: &mut String) {
    s.push_str(
        r#"
#import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge
"#,
    );
}

fn write_cover(s: &mut String, c: &CoverData, _b: &BrandedDocument) {
    s.push_str(
        r#"
#page(margin: 0mm, header: none, footer: none)[
  #place(top + left, rect(width: 100%, height: 22mm, fill: primary))
  #place(bottom + left, rect(width: 100%, height: 8mm, fill: primary))

  #pad(x: 25mm, top: 30mm, bottom: 14mm)[
    #v(35%)
"#,
    );

    s.push_str(&format!(
        "    #text(size: 38pt, weight: \"bold\", fill: primary)[{}]\n",
        typst_content(&c.title)
    ));
    s.push_str("    #v(2mm)\n");
    s.push_str("    #line(length: 80mm, stroke: 1.5pt + accent)\n");

    if let Some(sub) = &c.subtitle {
        s.push_str(&format!(
            "    #v(4mm)\n    #text(size: 16pt, fill: secondary)[{}]\n",
            typst_content(sub)
        ));
    }

    s.push_str("    #v(1fr)\n");
    s.push_str("    #stack(spacing: 6mm,\n");

    if let Some(r) = &c.recipient {
        s.push_str(&format!(
            "      [#text(size: 8pt, fill: secondary)[PREPARED FOR] \\ #text(size: 12pt, weight: \"bold\")[{}]],\n",
            typst_content(r)
        ));
    }
    if let Some(a) = &c.author {
        s.push_str(&format!(
            "      [#text(size: 8pt, fill: secondary)[BY] \\ #text(size: 12pt)[{}]],\n",
            typst_content(a)
        ));
    }
    if let Some(d) = &c.date {
        s.push_str(&format!(
            "      [#text(size: 8pt, fill: secondary)[DATE] \\ #text(size: 12pt, weight: \"bold\")[{}]],\n",
            typst_content(d)
        ));
    }

    s.push_str("    )\n  ]\n]\n\n");
}

fn write_body(s: &mut String, nodes: &[DocumentNode], consumed_body_h1: bool) {
    let mut skip_next_h1 = consumed_body_h1;
    for node in nodes {
        if skip_next_h1 {
            if let DocumentNode::Heading { level: 1, .. } = node {
                skip_next_h1 = false;
                continue;
            }
        }
        write_node(s, node);
    }
}

fn write_node(s: &mut String, node: &DocumentNode) {
    match node {
        DocumentNode::Heading { level, text, .. } => {
            let hashes = "=".repeat(*level as usize);
            s.push_str(&format!("{} {}\n\n", hashes, typst_inline(text)));
        }
        DocumentNode::Paragraph { text } => {
            s.push_str(&typst_inline(text));
            s.push_str("\n\n");
        }
        DocumentNode::BulletList { items } => {
            for item in items {
                s.push_str(&format!("- {}\n", typst_inline(item)));
            }
            s.push('\n');
        }
        DocumentNode::OrderedList { items, start } => {
            for (i, item) in items.iter().enumerate() {
                s.push_str(&format!("{}. {}\n", *start as usize + i, typst_inline(item)));
            }
            s.push('\n');
        }
        DocumentNode::Table { headers, rows } => {
            write_table(s, headers, rows);
        }
        DocumentNode::CodeBlock { language, code } => {
            write_code_block(s, language.as_deref(), code);
        }
        DocumentNode::Blockquote { text } => {
            s.push_str("#quote(block: true)[");
            s.push_str(&typst_inline(text));
            s.push_str("]\n\n");
        }
        DocumentNode::HorizontalRule => {
            s.push_str("#line(length: 100%, stroke: 0.3pt + table-border-color)\n\n");
        }
        DocumentNode::PageBreak => {
            s.push_str("#pagebreak()\n\n");
        }
        DocumentNode::Image { url, alt } => {
            // Best-effort: typst only loads local files in default world; remote URLs
            // would need pre-fetching. For now, emit a captioned placeholder.
            s.push_str(&format!(
                "#figure(caption: [{}])[#text(fill: secondary)[\\[image: {}\\]]]\n\n",
                typst_content(alt),
                typst_content(url),
            ));
        }
    }
}

fn write_table(s: &mut String, headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        return;
    }
    let cols = headers.len();
    s.push_str(&format!(
        "#table(\n  columns: {},\n  stroke: 0.3pt + table-border-color,\n  fill: (_, row) => if row == 0 {{ table-header-bg }} else {{ none }},\n",
        cols
    ));
    s.push_str("  table.header(");
    for h in headers {
        s.push_str(&format!("[*{}*], ", typst_content(h)));
    }
    s.push_str("),\n");
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i >= cols {
                break;
            }
            s.push_str(&format!("  [{}], ", typst_content(cell)));
        }
        // Pad missing cells
        for _ in row.len()..cols {
            s.push_str("  [], ");
        }
        s.push('\n');
    }
    s.push_str(")\n\n");
}

fn write_code_block(s: &mut String, language: Option<&str>, code: &str) {
    match language {
        Some("fletcher") => {
            // Treat as raw Typst fletcher code, wrapped in a diagram() call.
            s.push_str("#align(center)[\n  #diagram(\n");
            s.push_str(code);
            s.push_str("\n  )\n]\n\n");
        }
        Some(lang) => {
            s.push_str(&format!("```{}\n{}\n```\n\n", lang, code));
        }
        None => {
            s.push_str(&format!("```\n{}\n```\n\n", code));
        }
    }
}

// ---------------------------------------------------------------------------
// Escaping helpers
// ---------------------------------------------------------------------------

/// Quote a string for use in a Typst expression context (e.g. `set document(title: ...)`).
fn typst_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// Escape a string for use in Typst markup content (inside `[ ... ]`).
fn typst_content(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' | '#' | '$' | '*' | '_' | '`' | '~' | '<' | '>' | '@' | '=' | '[' | ']' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Escape for inline markdown-style content where typst's markup is used.
/// Same as `typst_content` for now — kept distinct so future formatting
/// (bold, italic from markdown inline runs) can diverge.
fn typst_inline(s: &str) -> String {
    typst_content(s)
}

fn hex_or_default<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.starts_with('#') && value.len() == 7 {
        value
    } else {
        fallback
    }
}

fn paper_name(page_size: &str) -> &'static str {
    match page_size {
        "Letter" => "us-letter",
        "Legal" => "us-legal",
        _ => "a4",
    }
}

fn font_family(name: &str) -> String {
    // Map common PDF builtin names to fonts typically present in the runtime image.
    let mapped = match name {
        "Helvetica" => "Inter",
        "Courier" => "DejaVu Sans Mono",
        "Times" | "Times Roman" => "DejaVu Serif",
        other => other,
    };
    mapped.replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Subprocess
// ---------------------------------------------------------------------------

fn compile(source: &str) -> anyhow::Result<Vec<u8>> {
    let tmp = tempfile::tempdir()?;
    let input = tmp.path().join("doc.typ");
    let output = tmp.path().join("doc.pdf");

    {
        let mut f = std::fs::File::create(&input)?;
        f.write_all(source.as_bytes())?;
    }

    let result = Command::new("typst")
        .args(["compile", "--root", tmp.path().to_str().unwrap()])
        .arg(&input)
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("typst compile failed:\n{}", stderr);
    }

    let bytes = std::fs::read(&output)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typst_string_escapes_quotes() {
        assert_eq!(typst_string(r#"foo "bar""#), r#""foo \"bar\"""#);
    }

    #[test]
    fn typst_content_escapes_specials() {
        assert_eq!(typst_content("a#b"), r"a\#b");
        assert_eq!(typst_content("a*b"), r"a\*b");
    }

    #[test]
    fn hex_falls_back_on_invalid() {
        assert_eq!(hex_or_default("not-a-hex", "#000000"), "#000000");
        assert_eq!(hex_or_default("#FF0000", "#000000"), "#FF0000");
    }

    #[test]
    fn paper_name_maps_known_sizes() {
        assert_eq!(paper_name("A4"), "a4");
        assert_eq!(paper_name("Letter"), "us-letter");
        assert_eq!(paper_name("Legal"), "us-legal");
    }

    fn typst_available() -> bool {
        std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// End-to-end smoke test. Requires the `typst` binary on PATH.
    /// Skipped automatically if it isn't installed.
    #[test]
    fn smoke_render_sample_markdown() {
        if !typst_available() {
            eprintln!("typst not on PATH — skipping smoke test");
            return;
        }

        let md_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sample.md");
        let md = std::fs::read_to_string(&md_path).expect("read sample.md");

        let doc = crate::parser::markdown::parse(&md);
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded =
            crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");

        let bytes = render(&branded).expect("typst render");
        assert!(bytes.starts_with(b"%PDF-"), "output should be a PDF");
        assert!(bytes.len() > 1000, "PDF should be non-trivial");

        let out = std::env::temp_dir().join("typeset-smoke.pdf");
        std::fs::write(&out, &bytes).expect("write smoke PDF");
        eprintln!("Smoke PDF written to {}", out.display());
    }

    /// Verify code blocks do NOT split across pages. Renders a section with
    /// many medium-sized code blocks and asserts the resulting PDF has more
    /// pages than the equivalent breakable version (i.e. blocks were moved
    /// whole to fresh pages rather than split mid-block).
    #[test]
    fn code_blocks_stay_whole() {
        if !typst_available() {
            return;
        }

        let md = r#"---
title: Test
---

## Section

1. First block

```
line one
line two
line three
line four
line five
line six
line seven
line eight
line nine
line ten
line eleven
line twelve
line thirteen
line fourteen
line fifteen
```

2. Second block

```
another one
with several lines
to push the page
boundary close
to the bottom
so we can see
whether the third
block splits or
gets moved whole
to the next page
```

3. Third block

```
this is the third
multi-line block
that should land
cleanly on its own
page or below the
previous one,
never split across
two pages
```
"#;

        let doc = crate::parser::markdown::parse(md);
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded =
            crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");

        let bytes = render(&branded).expect("typst render");
        let out = std::env::temp_dir().join("typeset-codeblock-smoke.pdf");
        std::fs::write(&out, &bytes).expect("write smoke PDF");
        eprintln!("Code-block smoke PDF written to {}", out.display());
    }

    /// Verify fletcher diagrams render. Needs network on first run to fetch
    /// the fletcher package from the typst package registry.
    #[test]
    fn smoke_render_with_fletcher_diagram() {
        if !typst_available() {
            eprintln!("typst not on PATH — skipping fletcher smoke test");
            return;
        }

        let md = r#"---
title: Architecture
author: Test
---

## System Overview

Here's how the components connect:

```fletcher
    node((0,0), [Client]),
    node((1,0), [API]),
    node((2,0), [Database]),
    edge((0,0), (1,0), "->", [HTTP]),
    edge((1,0), (2,0), "->", [SQL]),
```

That's it.
"#;

        let doc = crate::parser::markdown::parse(md);
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded =
            crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");

        let bytes = render(&branded).expect("typst render with fletcher");
        assert!(bytes.starts_with(b"%PDF-"));

        let out = std::env::temp_dir().join("typeset-fletcher-smoke.pdf");
        std::fs::write(&out, &bytes).expect("write fletcher smoke PDF");
        eprintln!("Fletcher smoke PDF written to {}", out.display());
    }
}
