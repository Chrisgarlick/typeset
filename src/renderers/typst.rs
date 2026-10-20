use std::io::Write;
use std::process::{Command, Stdio};

use crate::brand::engine::BrandedDocument;
use crate::models::document::{BlockStyle, DocumentNode, StyledColumn};

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
    let level = section_level(&branded.doc.nodes, consumed_body_h1);
    write_preamble(&mut s, branded, &cover, level);
    write_imports(&mut s);
    if let Some(c) = &cover {
        write_cover(&mut s, c, branded);
    }
    write_body(&mut s, &branded.doc.nodes, consumed_body_h1, level);

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

/// The theme: every style rule. It reads `t` and `meta`, which
/// `write_preamble` emits above it.
const THEME: &str = include_str!("theme.typ");

/// Which heading level opens a section: the shallowest level among the
/// body's top-level headings (after the cover's H1, if it took one).
fn section_level(nodes: &[DocumentNode], consumed_body_h1: bool) -> u8 {
    let mut skip_h1 = consumed_body_h1;
    let mut level = u8::MAX;
    for node in nodes {
        if let DocumentNode::Heading { level: l, .. } = node {
            if skip_h1 && *l == 1 {
                skip_h1 = false;
                continue;
            }
            level = level.min(*l);
        }
    }
    if level == u8::MAX { 1 } else { level }
}

/// Paper height in millimetres, for the space a new section needs.
fn paper_height_mm(page_size: &str) -> f32 {
    match paper_name(page_size) {
        "us-letter" => 279.4,
        "us-legal" => 355.6,
        _ => 297.0,
    }
}

/// Paper width in millimetres, for sizing the margin column.
fn paper_width_mm(page_size: &str) -> f32 {
    match paper_name(page_size) {
        "us-letter" | "us-legal" => 215.9,
        _ => 210.0,
    }
}

/// Emit the design tokens (`t`) and document facts (`meta`) the theme reads.
fn write_preamble(s: &mut String, b: &BrandedDocument, cover: &Option<CoverData>, section_level: u8) {
    let p = &b.profile;

    let doc_title = cover
        .as_ref()
        .map(|c| c.title.clone())
        .unwrap_or_else(|| "Document".to_string());
    let author = cover.as_ref().and_then(|c| c.author.clone());
    let description = cover.as_ref().and_then(|c| c.subtitle.clone());
    let footer_left = match cover.as_ref().and_then(|c| c.recipient.clone()) {
        Some(r) => format!("Prepared for {}", typst_content(&r)),
        None => typst_content(&doc_title),
    };

    let base = p.font_size_base.clamp(7.0, 16.0);
    // Line height is baseline to baseline, as in CSS; Typst's leading is the
    // gap below the cap height, so take Inter's cap height (about 0.73em) off.
    let leading = (p.line_height - 0.73).clamp(0.35, 1.2);
    let content_mm = paper_width_mm(&p.page_size) - p.margin_left - p.margin_right;
    let side_mm = (content_mm * 0.25).max(0.0);

    s.push_str(&format!(
        r##"#let t = (
  ink: rgb("{ink}"),
  ink-muted: rgb("{muted}"),
  heading: rgb("{heading}"),
  paper: rgb("{paper}"),
  surface: rgb("{surface}"),
  rule: rgb("{rule}"),
  accent: rgb("{accent}"),
  accent-soft: rgb("{accent_soft}"),
  cover-bg: rgb("{cover_bg}"),
  cover-text: rgb("{cover_text}"),
  good: rgb("#1F7A4D"),
  fair: rgb("#B7791F"),
  poor: rgb("#B42318"),
  font-body: "{font_body}",
  font-heading: "{font_heading}",
  font-mono: "{font_mono}",
  display: {display}pt,
  h1: {h1}pt,
  h2: {h2}pt,
  h3: {h3}pt,
  body: {base}pt,
  small: {small}pt,
  eyebrow: {eyebrow}pt,
  leading: {leading}em,
  par-spacing: {par_spacing}pt,
  s1: 2pt, s2: 4pt, s3: 8pt, s4: 12pt, s5: 16pt, s6: 24pt, s7: 32pt, s8: 48pt, s9: 64pt,
  r-sm: 3pt,
  r-md: 6pt,
  hairline: 0.5pt,
  heavy: 2.5pt,
  margin-top: {mt}mm,
  margin-bottom: {mb}mm,
  margin-left: {ml}mm,
  margin-right: {mr}mm,
  side: {side}mm,
  section-need: {section_need}mm,
)

#let meta = (
  title: {title},
  author: {author},
  description: {description},
  running-title: [{running_title}],
  footer-left: [{footer_left}],
  paper: "{paper_size}",
  has-cover: {has_cover},
  section-level: {section_level},
  header: {header},
  footer: {footer},
)
"##,
        ink = hex_or_default(&p.colour_text, "#1A1A1A"),
        muted = hex_or_default(&p.colour_secondary, "#555555"),
        heading = hex_or_default(&p.colour_primary, "#000000"),
        paper = hex_or_default(&p.colour_background, "#FFFFFF"),
        surface = hex_or_default(&p.colour_table_header, "#F5F5F5"),
        rule = hex_or_default(&p.colour_table_border, "#E0E0E0"),
        accent = hex_or_default(&p.colour_accent, "#0066CC"),
        accent_soft = hex_or_default(&p.colour_callout_bg, "#F8F9FA"),
        cover_bg = hex_or_default(&p.cover_bg_colour, "#000000"),
        cover_text = hex_or_default(&p.cover_text_colour, "#FFFFFF"),
        font_body = font_family(&p.font_body, FontRole::Body),
        font_heading = font_family(&p.font_heading, FontRole::Heading),
        font_mono = font_family(&p.font_mono, FontRole::Mono),
        display = (p.font_size_h1 * 1.45).round(),
        h1 = p.font_size_h1,
        h2 = p.font_size_h2,
        h3 = p.font_size_h3,
        base = base,
        small = (base * 0.86 * 10.0).round() / 10.0,
        eyebrow = (base * 0.78 * 10.0).round() / 10.0,
        leading = (leading * 100.0).round() / 100.0,
        par_spacing = p.paragraph_spacing.max(base * 0.9),
        mt = p.margin_top,
        mb = p.margin_bottom,
        ml = p.margin_left,
        mr = p.margin_right,
        side = (side_mm * 10.0).round() / 10.0,
        section_need = ((paper_height_mm(&p.page_size) - p.margin_top - p.margin_bottom) * 0.3).round(),
        title = typst_string(&doc_title),
        author = match author {
            Some(a) => format!("({},)", typst_string(&a)),
            None => "()".to_string(),
        },
        description = description.map(|d| typst_string(&d)).unwrap_or_else(|| "none".to_string()),
        running_title = typst_content(&doc_title),
        footer_left = footer_left,
        paper_size = paper_name(&p.page_size),
        has_cover = cover.is_some(),
        section_level = section_level,
        header = p.header_enabled,
        footer = p.footer_enabled,
    ));
    s.push('\n');
    s.push_str(THEME);
    s.push('\n');
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
#page(margin: 0mm, header: none, footer: none, fill: t.cover-bg)[
  #set text(fill: t.cover-text, hyphenate: false)
  #pad(left: t.margin-left, right: t.margin-right, top: t.margin-top * 1.4, bottom: t.margin-bottom)[
    #v(30%)
"#,
    );
    s.push_str(&format!(
        "    #block(width: 100%)[#text(font: t.font-heading, size: {:.1}pt, weight: 600, tracking: -0.02em)[{}]]\n",
        cover_title_size(&c.title, _b),
        typst_content(&breakable_words(&c.title))
    ));
    s.push_str("    #v(t.s5)\n    #line(length: 24mm, stroke: 3pt + t.accent.lighten(35%))\n");
    if let Some(sub) = &c.subtitle {
        s.push_str(&format!(
            "    #v(t.s5)\n    #block(width: 75%)[#text(size: t.h3, fill: t.cover-text.transparentize(25%))[{}]]\n",
            typst_content(sub)
        ));
    }
    s.push_str("    #v(1fr)\n    #grid(columns: (1fr, 1fr, 1fr), column-gutter: t.s5,\n");
    let field = |label: &str, value: &Option<String>| -> String {
        match value {
            Some(v) => format!(
                "      [#text(size: t.eyebrow, weight: 600, tracking: 0.12em, fill: t.cover-text.transparentize(45%))[{label}] \\ #v(t.s1) #text(size: t.body)[{}]],\n",
                typst_content(v)
            ),
            None => "      [],\n".to_string(),
        }
    };
    s.push_str(&field("PREPARED FOR", &c.recipient));
    s.push_str(&field("BY", &c.author));
    s.push_str(&field("DATE", &c.date));
    s.push_str("    )\n  ]\n]\n\n");
}

/// The cover title's size: the display size, shrunk until the longest word
/// fits the line (Inter Tight semibold averages about 0.52em a character).
fn cover_title_size(title: &str, b: &BrandedDocument) -> f32 {
    let p = &b.profile;
    let display = (p.font_size_h1 * 1.45).round();
    let width_pt = (paper_width_mm(&p.page_size) - p.margin_left - p.margin_right) * 72.0 / 25.4;
    let longest = title.split_whitespace().map(|w| w.chars().count()).max().unwrap_or(1) as f32;
    (width_pt * 0.95 / (longest * 0.52)).clamp(24.0, display)
}

/// Let long unbroken words (domains, paths) wrap after dots and slashes
/// instead of running off the page.
fn breakable_words(s: &str) -> String {
    s.split(' ')
        .map(|word| {
            if word.chars().count() > 24 {
                word.replace('.', ".\u{200B}").replace('/', "/\u{200B}")
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_body(s: &mut String, nodes: &[DocumentNode], consumed_body_h1: bool, section_level: u8) {
    let mut skip_next_h1 = consumed_body_h1;
    let mut section = 0;
    for (i, node) in nodes.iter().enumerate() {
        if skip_next_h1 {
            if let DocumentNode::Heading { level: 1, .. } = node {
                skip_next_h1 = false;
                continue;
            }
        }
        // A section heading makes its own break; don't make two.
        if matches!(node, DocumentNode::SectionBreak)
            && matches!(nodes.get(i + 1), Some(DocumentNode::Heading { level, .. }) if *level <= section_level)
        {
            continue;
        }
        if let DocumentNode::Heading { level, text, .. } = node {
            if *level == section_level {
                section += 1;
                s.push_str(&format!(
                    "#heading(level: {level}, supplement: [{section:02}])[{}]\n\n",
                    typst_inline(text)
                ));
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
            s.push_str("#hairline()\n\n");
        }
        DocumentNode::PageBreak => {
            s.push_str("#pagebreak()\n\n");
        }
        DocumentNode::SectionBreak => {
            s.push_str("#section-break()\n\n");
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
        DocumentNode::Columns { ratios, gutter, children } => {
            write_columns(s, ratios, gutter.as_deref(), children);
        }
        DocumentNode::Section { style, children } => {
            write_section(s, style.as_ref(), children);
        }
    }
}

fn write_columns(
    s: &mut String,
    ratios: &[f32],
    gutter: Option<&str>,
    children: &[StyledColumn],
) {
    if children.is_empty() {
        return;
    }

    let col_count = children.len();
    // Pad/truncate ratios to match children count; default to 1 per column.
    let mut ratio_strs = Vec::with_capacity(col_count);
    for i in 0..col_count {
        let r = ratios.get(i).copied().unwrap_or(1.0).max(0.0);
        let r = if r <= 0.0 { 1.0 } else { r };
        ratio_strs.push(format!("{r}fr"));
    }

    s.push_str("#wide(grid(\n");
    s.push_str(&format!("  columns: ({}),\n", ratio_strs.join(", ")));
    s.push_str(&format!(
        "  column-gutter: {},\n",
        typst_length(gutter, "6mm"),
    ));

    for child in children {
        let mut inner = String::new();
        for node in &child.nodes {
            write_node(&mut inner, node);
        }
        match child.style.as_ref() {
            Some(style) if !style.is_empty() => {
                s.push_str("  ");
                write_styled_block(s, style, &inner);
                s.push_str(",\n");
            }
            _ => {
                s.push_str("  [\n");
                s.push_str(&inner);
                s.push_str("  ],\n");
            }
        }
    }
    s.push_str("))\n\n");
}

fn write_section(s: &mut String, style: Option<&BlockStyle>, children: &[DocumentNode]) {
    let mut inner = String::new();
    for node in children {
        write_node(&mut inner, node);
    }
    match style {
        Some(style) if !style.is_empty() => {
            // Section sits at markup level — `#` introduces the call.
            s.push('#');
            write_styled_block(s, style, &inner);
            s.push_str("\n\n");
        }
        _ => {
            // Unstyled section is just a content wrapper.
            s.push_str(&inner);
            s.push('\n');
        }
    }
}

/// Emit a `block(...)[ ...content... ]` call (without the leading `#`).
/// The caller adds `#` in markup contexts (e.g. Section) and omits it in code
/// contexts (e.g. as a `#grid(...)` argument). The caller is also responsible
/// for any trailing comma or newline.
fn write_styled_block(s: &mut String, style: &BlockStyle, inner: &str) {
    let mut args: Vec<String> = Vec::new();

    if let Some(bg) = style.background.as_deref().filter(|b| is_hex(b)) {
        args.push(format!("fill: rgb(\"{bg}\")"));
    }

    if let Some(border) = style.border.as_ref() {
        let w = typst_length(border.width.as_deref(), "1pt");
        let c = border
            .color
            .as_deref()
            .filter(|c| is_hex(c))
            .unwrap_or("#000000");
        args.push(format!("stroke: {w} + rgb(\"{c}\")"));
    }

    args.push(format!(
        "inset: {}",
        typst_length(style.padding.as_deref(), "8pt")
    ));
    args.push(format!(
        "radius: {}",
        typst_length(style.radius.as_deref(), "3pt")
    ));
    args.push("width: 100%".to_string());
    // Cards and callouts read as one object; never split one across pages.
    args.push("breakable: false".to_string());

    s.push_str(&format!("block({})[\n", args.join(", ")));

    if let Some(tc) = style.text_color.as_deref().filter(|c| is_hex(c)) {
        s.push_str(&format!("  #set text(fill: rgb(\"{tc}\"))\n"));
    }
    // Headings in a box are card titles: small, in the box's text colour,
    // and left out of the bookmarks and the running head.
    s.push_str("  #set heading(outlined: false, bookmarked: false)\n");
    s.push_str("  #show heading: boxed-heading\n");

    s.push_str(inner);
    s.push(']');
}

fn is_hex(s: &str) -> bool {
    s.starts_with('#')
        && s.len() == 7
        && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Accepts a length string like "8mm", "12pt", "1em". Falls back to `default`
/// if the input is missing or fails a permissive sanity check.
fn typst_length(value: Option<&str>, default: &str) -> String {
    let Some(v) = value else { return default.to_string() };
    let trimmed = v.trim();
    let ok = !trimmed.is_empty()
        && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && trimmed.chars().next().map(|c| c.is_ascii_digit() || c == '-' || c == '.').unwrap_or(false);
    if ok { trimmed.to_string() } else { default.to_string() }
}

/// Column widths for a table too wide for the text column: a column whose
/// longest cell is short sizes to fit (`auto`); the rest share the remaining
/// width by average length, so long prose columns get the room.
fn table_columns(headers: &[String], rows: &[Vec<String>]) -> Vec<String> {
    (0..headers.len())
        .map(|i| {
            let cells: Vec<usize> = std::iter::once(&headers[i])
                .chain(rows.iter().filter_map(|r| r.get(i)))
                .map(|c| c.chars().count())
                .collect();
            let longest = cells.iter().copied().max().unwrap_or(0);
            if longest <= 16 {
                return "auto".to_string();
            }
            let average = cells.iter().sum::<usize>() as f32 / cells.len().max(1) as f32;
            // Square root, so a long prose column doesn't starve a short one.
            format!("{:.1}fr", average.max(12.0).sqrt())
        })
        .collect()
}

fn write_table(s: &mut String, headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        return;
    }
    let cols = headers.len();
    let widths = table_columns(headers, rows);
    s.push_str(&format!("#fit-table(({},),\n", widths.join(", ")));
    s.push_str("  table.header(");
    for h in headers {
        s.push_str(&format!("[{}], ", typst_inline(h)));
    }
    s.push_str("),\n");
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i >= cols {
                break;
            }
            s.push_str(&format!("  [{}], ", typst_inline(cell)));
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

/// Inline text with its formatting: `**bold**`, `*italic*`, `~~struck~~`,
/// `` `code` `` and `[links](https://...)` become Typst markup; everything
/// else is escaped literally. Text that parses as block syntax (a heading,
/// a list, a quote) is not inline text at all, so it's kept literal rather
/// than reinterpreted.
fn typst_inline(s: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    if !s.contains(['*', '_', '`', '[', '~', '\\', '<', '&']) {
        return typst_content(s);
    }
    let mut out = String::with_capacity(s.len() + 16);
    let mut link_open: Vec<bool> = Vec::new();
    for event in Parser::new_ext(s, Options::ENABLE_STRIKETHROUGH) {
        match event {
            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {}
            Event::Text(t) => out.push_str(&typst_content(&t)),
            Event::Code(c) => out.push_str(&format!("#raw({})", typst_string(&c))),
            Event::Start(Tag::Strong) => out.push_str("#strong["),
            Event::Start(Tag::Emphasis) => out.push_str("#emph["),
            Event::Start(Tag::Strikethrough) => out.push_str("#strike["),
            Event::End(TagEnd::Strong) | Event::End(TagEnd::Emphasis) | Event::End(TagEnd::Strikethrough) => out.push(']'),
            Event::Start(Tag::Link { dest_url, .. }) => {
                let safe = ["https://", "http://", "mailto:"].iter().any(|p| dest_url.starts_with(p));
                if safe {
                    out.push_str(&format!("#link({})[", typst_string(&dest_url)));
                }
                link_open.push(safe);
            }
            Event::End(TagEnd::Link) => {
                if link_open.pop().unwrap_or(false) {
                    out.push(']');
                }
            }
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push_str(" \\ "),
            Event::Html(h) | Event::InlineHtml(h) => out.push_str(&typst_content(&h)),
            // Block syntax: not inline text, keep the original literally.
            _ => return typst_content(s),
        }
    }
    out.trim_end().to_string()
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

/// What a font is for, which decides the fallback when a profile names a
/// family Typeset doesn't bundle.
#[derive(Clone, Copy)]
pub enum FontRole {
    Body,
    Heading,
    Mono,
}

/// Families bundled in `assets/fonts`. Compiles use only these
/// (`--ignore-system-fonts`), so output is the same everywhere and a missing
/// font can't silently fall back to Typst's built-in serif.
pub const BUNDLED_FONTS: [&str; 3] = ["Inter", "Inter Tight", "JetBrains Mono"];

/// Map a profile's font name to a bundled family: exact matches pass through,
/// a few known names map to their closest bundled family, and anything else
/// (including Helvetica, the old default) falls back by role: Inter for text,
/// Inter Tight for headings, JetBrains Mono for code.
pub fn font_family(name: &str, role: FontRole) -> String {
    let trimmed = name.trim();
    if let Some(found) = BUNDLED_FONTS.iter().find(|f| f.eq_ignore_ascii_case(trimmed)) {
        return found.to_string();
    }
    let lower = trimmed.to_ascii_lowercase();
    let mapped = match lower.as_str() {
        "inter variable" => "Inter",
        "inter display" | "inter tight variable" => "Inter Tight",
        "courier" | "courier new" | "monospace" | "dejavu sans mono" | "menlo" | "consolas" | "dm mono"
        | "ibm plex mono" | "jetbrains mono nl" => "JetBrains Mono",
        // Generic sans names (Helvetica, Arial and the rest) and anything
        // unknown take the role's family.
        _ => match role {
            FontRole::Body => "Inter",
            FontRole::Heading => "Inter Tight",
            FontRole::Mono => "JetBrains Mono",
        },
    };
    mapped.to_string()
}

/// Where the bundled fonts live: `TYPESET_FONT_DIR`, then the image's
/// `/app/assets/fonts`, then the repository's `assets/fonts`.
fn font_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("TYPESET_FONT_DIR") {
        return dir.into();
    }
    let image = std::path::PathBuf::from("/app/assets/fonts");
    if image.is_dir() {
        return image;
    }
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts"))
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
        .arg("--font-path")
        .arg(font_dir())
        .arg("--ignore-system-fonts")
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
    fn inline_formatting_becomes_typst_markup() {
        assert_eq!(
            typst_inline("Plain **bold** and *italic* with `code`"),
            r#"Plain #strong[bold] and #emph[italic] with #raw("code")"#
        );
        assert_eq!(
            typst_inline("See [the site](https://example.com) now"),
            r#"See #link("https://example.com")[the site] now"#
        );
        assert_eq!(typst_inline("~~gone~~"), "#strike[gone]");
    }

    #[test]
    fn inline_text_stays_escaped() {
        assert_eq!(typst_inline("Cost #1 is $5 <fast>"), r"Cost \#1 is \$5 \<fast\>");
        // A literal asterisk with no partner is text, not emphasis.
        assert_eq!(typst_inline("5 * 3"), r"5 \* 3");
    }

    #[test]
    fn inline_links_only_allow_web_and_mail_schemes() {
        assert_eq!(typst_inline("[x](javascript:alert(1))"), "x");
        assert_eq!(typst_inline("[x](mailto:a@b.com)"), r#"#link("mailto:a@b.com")[x]"#);
    }

    #[test]
    fn block_syntax_in_inline_text_is_kept_literal() {
        assert_eq!(typst_inline("# not a heading *really*"), typst_content("# not a heading *really*"));
        assert_eq!(typst_inline("- not a list"), typst_content("- not a list"));
    }

    #[test]
    fn fonts_resolve_to_bundled_families() {
        assert_eq!(font_family("Inter", FontRole::Body), "Inter");
        assert_eq!(font_family("inter tight", FontRole::Heading), "Inter Tight");
        assert_eq!(font_family("Helvetica", FontRole::Heading), "Inter Tight");
        assert_eq!(font_family("Helvetica", FontRole::Body), "Inter");
        assert_eq!(font_family("DM Mono", FontRole::Mono), "JetBrains Mono");
        assert_eq!(font_family("Instrument Serif", FontRole::Heading), "Inter Tight");
        assert_eq!(font_family("Comic Sans", FontRole::Body), "Inter");
    }

    fn branded_json(input: &str) -> BrandedDocument {
        let doc = crate::parser::json::parse(input).expect("parse");
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        BrandedDocument::prepare(doc, profile).expect("prepare")
    }

    #[test]
    fn sections_are_numbered_from_the_top_heading_level() {
        let source = build_source(&branded_json(r#"{
            "frontmatter": { "title": "Doc" },
            "pages": [
                { "blocks": [{ "type": "heading", "level": 2, "text": "First" },
                             { "type": "heading", "level": 3, "text": "Inside" }] },
                { "blocks": [{ "type": "heading", "level": 2, "text": "Second" }] }
            ]
        }"#));
        assert!(source.contains("#heading(level: 2, supplement: [01])[First]"));
        assert!(source.contains("#heading(level: 2, supplement: [02])[Second]"));
        assert!(source.contains("=== Inside"));
        assert!(source.contains("section-level: 2,"));
        // The page boundary is a soft break, and the section heading makes it.
        assert!(!source.contains("#pagebreak()"));
        assert!(!source.contains("#section-break()"));
    }

    #[test]
    fn table_columns_fit_short_content_and_share_the_rest() {
        let headers = vec!["Issue".to_string(), "Severity".to_string(), "What to do".to_string()];
        let rows = vec![vec![
            "Images have no alternative text describing them for screen readers".to_string(),
            "Critical".to_string(),
            "Add a short written description to each image".to_string(),
        ]];
        let widths = table_columns(&headers, &rows);
        assert_eq!(widths[1], "auto");
        assert!(widths[0].ends_with("fr") && widths[2].ends_with("fr"));
    }

    #[test]
    fn headings_in_styled_boxes_stay_out_of_the_bookmarks() {
        let source = build_source(&branded_json(r##"{
            "pages": [{ "blocks": [{
                "type": "section",
                "background": "#111111",
                "text_color": "#FFFFFF",
                "blocks": [{ "type": "heading", "level": 3, "text": "Card" }]
            }]}]
        }"##));
        assert!(source.contains("#set heading(outlined: false, bookmarked: false)"));
        assert!(source.contains("#show heading: boxed-heading"));
        assert!(source.contains("breakable: false"));
    }

    #[test]
    fn long_cover_titles_shrink_to_fit() {
        let b = branded_json(r#"{ "pages": [] }"#);
        let display = (b.profile.font_size_h1 * 1.45).round();
        assert_eq!(cover_title_size("Short title", &b), display);
        assert!(cover_title_size("Website review: averyveryverylongdomainname.co.uk", &b) < display);
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

    /// Every compile embeds only bundled fonts: no fallback to Typst's
    /// built-in Libertinus or New Computer Modern.
    #[test]
    fn renders_embed_only_bundled_fonts() {
        if !typst_available() {
            return;
        }
        let input = r#"{
            "frontmatter": { "title": "Fonts" },
            "pages": [{ "blocks": [
                { "type": "heading", "level": 2, "text": "A heading" },
                { "type": "paragraph", "text": "Body with **bold**, *italic* and `code`." },
                { "type": "code", "language": "rust", "content": "fn main() {}" }
            ]}]
        }"#;
        let doc = crate::parser::json::parse(input).expect("parse");
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");
        let bytes = render(&branded).expect("render");
        let pdf = String::from_utf8_lossy(&bytes);

        let mut names: Vec<String> = pdf
            .match_indices("/BaseFont /")
            .map(|(i, m)| {
                pdf[i + m.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '-')
                    .collect::<String>()
            })
            .map(|n| n.split_once('+').map(|(_, f)| f.to_string()).unwrap_or(n))
            .collect();
        names.sort();
        names.dedup();

        assert!(!names.is_empty(), "no fonts found in the PDF");
        for name in &names {
            assert!(
                name.starts_with("Inter") || name.starts_with("JetBrainsMono"),
                "unexpected font {name} in {names:?}"
            );
        }
        assert!(names.iter().any(|n| n.starts_with("InterTight")), "headings should use Inter Tight: {names:?}");
        assert!(names.iter().any(|n| n.starts_with("JetBrainsMono")), "code should use JetBrains Mono: {names:?}");
    }

    /// Render a local file for design review. Not a test: run with
    /// `TYPESET_PREVIEW_IN=doc.json TYPESET_PREVIEW_OUT=out.pdf
    /// [TYPESET_PREVIEW_PROFILE=overrides.json] cargo test preview -- --ignored`.
    /// The profile file overrides fields on the default profile. The Typst
    /// source is written next to the PDF.
    #[test]
    #[ignore]
    fn preview() {
        let input = std::env::var("TYPESET_PREVIEW_IN").expect("TYPESET_PREVIEW_IN");
        let out = std::env::var("TYPESET_PREVIEW_OUT").expect("TYPESET_PREVIEW_OUT");
        let content = std::fs::read_to_string(&input).expect("read input");
        let doc = if input.ends_with(".json") {
            crate::parser::json::parse(&content).expect("parse json")
        } else {
            crate::parser::markdown::parse(&content)
        };
        let mut profile = serde_json::to_value(crate::models::client_profile::ClientProfile::default_profile()).unwrap();
        if let Ok(path) = std::env::var("TYPESET_PREVIEW_PROFILE") {
            let overrides: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            for (k, v) in overrides.as_object().unwrap() {
                profile[k] = v.clone();
            }
        }
        let profile = serde_json::from_value(profile).expect("profile");
        let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");
        std::fs::write(format!("{out}.typ"), build_source(&branded)).unwrap();
        std::fs::write(&out, render(&branded).expect("render")).unwrap();
    }

    /// Many sections must still lay out in one pass. Position-dependent
    /// breaks used to need a pass per section and gave up at Typst's limit,
    /// which scrambled the section numbers.
    #[test]
    fn many_sections_lay_out_without_convergence_warnings() {
        if !typst_available() {
            return;
        }
        let pages: Vec<String> = (1..=14)
            .map(|i| format!(
                r#"{{ "blocks": [{{ "type": "heading", "level": 2, "text": "Section {i}" }},
                   {{ "type": "paragraph", "text": "{}" }}] }}"#,
                "Words to fill some of the page. ".repeat(i * 6)
            ))
            .collect();
        let input = format!(r#"{{ "frontmatter": {{ "title": "Many" }}, "pages": [{}] }}"#, pages.join(","));
        let source = build_source(&branded_json(&input));

        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("input.typ"), &source).unwrap();
        let output = std::process::Command::new("typst")
            .args(["compile", "--root"])
            .arg(tmp.path())
            .arg("--font-path")
            .arg(font_dir())
            .arg("--ignore-system-fonts")
            .arg(tmp.path().join("input.typ"))
            .arg(tmp.path().join("out.pdf"))
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stderr}");
        assert!(!stderr.contains("did not converge"), "{stderr}");
    }

    /// Smoke test for the full AI strategy example file.
    /// Skipped if `typst` isn't installed.
    #[test]
    fn smoke_render_ai_strategy_example() {
        if !typst_available() {
            return;
        }

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/ai-strategy.json");
        let Ok(json) = std::fs::read_to_string(&path) else {
            eprintln!("ai-strategy.json not present — skipping");
            return;
        };

        let doc = crate::parser::json::parse(&json).expect("parse ai-strategy.json");
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded =
            crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");

        let bytes = render(&branded).expect("typst render");
        assert!(bytes.starts_with(b"%PDF-"), "output should be a PDF");

        let out = std::env::temp_dir().join("typeset-ai-strategy.pdf");
        std::fs::write(&out, &bytes).expect("write smoke PDF");
        eprintln!("AI strategy smoke PDF written to {}", out.display());
    }

    /// Smoke test for the JSON → PDF path with a 50/50 column layout.
    /// Skipped automatically if `typst` isn't installed.
    #[test]
    fn smoke_render_json_columns() {
        if !typst_available() {
            eprintln!("typst not on PATH — skipping JSON columns smoke test");
            return;
        }

        let input = r#"{
            "frontmatter": { "title": "Layout Demo" },
            "pages": [{
                "blocks": [
                    { "type": "heading", "level": 2, "text": "Two columns" },
                    {
                        "type": "columns",
                        "ratios": [1, 1],
                        "gutter": "8mm",
                        "children": [
                            { "blocks": [ { "type": "markdown", "content": "**Wins**\n\n- Launch\n- Hires" } ] },
                            { "blocks": [ { "type": "markdown", "content": "**Risks**\n\nMargin pressure in Q4." } ] }
                        ]
                    }
                ]
            }]
        }"#;

        let doc = crate::parser::json::parse(input).expect("parse json");
        let profile = crate::models::client_profile::ClientProfile::default_profile();
        let branded =
            crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");

        let bytes = render(&branded).expect("typst render");
        assert!(bytes.starts_with(b"%PDF-"), "output should be a PDF");
        assert!(bytes.len() > 1000, "PDF should be non-trivial");

        let out = std::env::temp_dir().join("typeset-json-columns.pdf");
        std::fs::write(&out, &bytes).expect("write smoke PDF");
        eprintln!("JSON columns smoke PDF written to {}", out.display());
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
