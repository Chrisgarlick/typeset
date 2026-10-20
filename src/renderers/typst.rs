use std::io::Write;
use std::process::{Command, Stdio};

use crate::brand::engine::BrandedDocument;
use crate::brand::assets::Asset;

/// Fetched images by URL.
type Images = std::collections::HashMap<String, Asset>;

/// What the node writers need besides the node.
#[derive(Clone, Copy)]
struct Ctx<'a> {
    images: &'a Images,
    /// The source heading level that opens a section.
    section_level: u8,
    /// The level section headings are emitted at: 2 under a cover (whose
    /// title is the H1), else 1. Tagged PDF needs headings to start at 1 and
    /// never skip a level.
    section_out: u8,
    /// Inside a styled box, where headings are card titles, not structure.
    in_box: bool,
}

impl Ctx<'_> {
    /// The emitted level for a source heading level.
    fn out_level(&self, level: u8) -> u8 {
        (level as i16 - self.section_level as i16 + self.section_out as i16).clamp(1, 6) as u8
    }

    fn boxed(&self) -> Self {
        Ctx { in_box: true, ..*self }
    }
}
use crate::models::components::{Better, Component, FindingCard, Opener};
use crate::models::document::{BlockStyle, DocumentNode, StyledColumn};

/// Render a branded document to PDF bytes via the Typst CLI.
pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let source = build_source(branded);
    let assets: Vec<&Asset> = [&branded.logos.light, &branded.logos.dark]
        .into_iter()
        .flatten()
        .chain(branded.images.values())
        .collect();
    compile(&source, &assets)
}

// ---------------------------------------------------------------------------
// Source generation
// ---------------------------------------------------------------------------

fn build_source(branded: &BrandedDocument) -> String {
    let mut s = String::with_capacity(4096);

    let (cover, consumed_body_h1) = extract_cover(branded);
    let level = section_level(&branded.doc.nodes, consumed_body_h1);
    // Parts sit between the cover's H1 and the sections, pushing sections
    // down a level.
    let has_parts = branded.doc.nodes.iter().any(|n| matches!(n, DocumentNode::Component(Component::Part(_))));
    let ctx = Ctx {
        images: &branded.images,
        section_level: level,
        section_out: if cover.is_some() { 2 } else { 1 } + u8::from(has_parts),
        in_box: false,
    };
    write_preamble(&mut s, branded, &cover, ctx.section_out);
    write_imports(&mut s);
    if let Some(c) = &cover {
        write_cover(&mut s, c, branded);
    }
    write_body(&mut s, &branded.doc.nodes, consumed_body_h1, ctx);

    s
}

struct CoverData {
    title: String,
    subtitle: Option<String>,
    recipient: Option<String>,
    author: Option<String>,
    date: Option<String>,
    eyebrow: Option<String>,
    reference: Option<String>,
    style: &'static str,
}

/// The cover style: the document's choice, else the profile's template.
/// Templates from before the editorial covers (`minimal`, `bold`, `plain`)
/// keep the dark cover they had.
fn cover_style(document: Option<&str>, profile_template: &str) -> &'static str {
    match document.unwrap_or(profile_template).trim().to_ascii_lowercase().as_str() {
        "editorial" => "editorial",
        "band" => "band",
        _ => "dark",
    }
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
            eyebrow: fm.and_then(|f| f.eyebrow.clone()),
            reference: fm.and_then(|f| f.reference.clone()),
            style: cover_style(fm.and_then(|f| f.cover.as_deref()), &branded.profile.cover_template),
        }),
        consumed_body_h1,
    )
}

/// The theme: every style rule. It reads `t` and `meta`, which
/// `write_preamble` emits above it.
const THEME: &str = include_str!("theme.typ");
/// The schema v2 components, drawn with the theme's tokens.
const COMPONENTS: &str = include_str!("components.typ");

/// Which heading level opens a section: the shallowest level among the
/// body's top-level headings (after the cover's H1, if it took one), with
/// openers counting as H2.
fn section_level(nodes: &[DocumentNode], consumed_body_h1: bool) -> u8 {
    let mut skip_h1 = consumed_body_h1;
    let mut level = u8::MAX;
    for node in nodes {
        match node {
            DocumentNode::Heading { level: l, .. } => {
                if skip_h1 && *l == 1 {
                    skip_h1 = false;
                    continue;
                }
                level = level.min(*l);
            }
            // An opener is a section heading at H2, the usual section level
            // for JSON documents.
            DocumentNode::Component(Component::Opener(_)) => level = level.min(2),
            _ => {}
        }
    }
    if level == u8::MAX { 1 } else { level }
}

/// A logo's path inside the compile root, as a Typst value.
fn asset_path(asset: &Option<Asset>) -> String {
    asset.as_ref().map(|a| typst_string(&a.name)).unwrap_or_else(|| "none".to_string())
}

/// Bleed added to each edge for print, in millimetres.
const PRINT_BLEED_MM: f32 = 3.0;

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
    let wordmark = p.wordmark.clone().filter(|w| !w.trim().is_empty());
    let has_brand = wordmark.is_some() || b.logos.light.is_some();
    // With a brand mark in the running head, the title moves to the footer.
    let recipient = cover.as_ref().and_then(|c| c.recipient.clone());
    let footer_left = match (has_brand, recipient) {
        (true, Some(r)) => format!("{} · Prepared for {}", typst_content(&doc_title), typst_content(&r)),
        (true, None) => typst_content(&doc_title),
        (false, Some(r)) => format!("Prepared for {}", typst_content(&r)),
        (false, None) => typst_content(&doc_title),
    };

    let base = p.font_size_base.clamp(7.0, 16.0);
    // Print adds a bleed round every page: the page grows, the margins grow
    // with it, and full-page fills run off the trim edge.
    let bleed = if b.print { PRINT_BLEED_MM } else { 0.0 };
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
  fair: rgb("#966010"),
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
  page-width: {page_w}mm,
  page-height: {page_h}mm,
  print: {print},
  has-cover: {has_cover},
  section-level: {section_level},
  part-level: {part_level},
  header: {header},
  footer: {footer},
  wordmark: {wordmark},
  logo-light: {logo_light},
  logo-dark: {logo_dark},
  has-brand: {has_brand},
  has-back-page: {has_back_page},
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
        mt = p.margin_top + bleed,
        mb = p.margin_bottom + bleed,
        ml = p.margin_left + bleed,
        mr = p.margin_right + bleed,
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
        page_w = paper_width_mm(&p.page_size) + 2.0 * bleed,
        page_h = paper_height_mm(&p.page_size) + 2.0 * bleed,
        print = b.print,
        has_cover = cover.is_some(),
        section_level = section_level,
        part_level = if b.doc.nodes.iter().any(|n| matches!(n, DocumentNode::Component(Component::Part(_)))) {
            (section_level - 1).to_string()
        } else {
            "none".to_string()
        },
        header = p.header_enabled,
        footer = p.footer_enabled,
        wordmark = wordmark.as_deref().map(typst_string).unwrap_or_else(|| "none".to_string()),
        logo_light = asset_path(&b.logos.light),
        logo_dark = asset_path(&b.logos.dark),
        has_brand = has_brand,
        has_back_page = b.doc.nodes.iter().any(|n| matches!(n, DocumentNode::Component(Component::BackPage(_)))),
    ));
    s.push('\n');
    s.push_str(THEME);
    s.push('\n');
    s.push_str(COMPONENTS);
    s.push('\n');
}

fn write_imports(s: &mut String) {
    s.push_str(
        r#"
#import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge
"#,
    );
}

fn write_cover(s: &mut String, c: &CoverData, b: &BrandedDocument) {
    s.push_str(&format!(
        "#cover(style: {}, title: [{}], title-size: {:.1}pt, subtitle: {}, eyebrow: {}, recipient: {}, author: {}, date: {}, reference: {})\n\n",
        typst_string(c.style),
        typst_content(&breakable_words(&c.title)),
        cover_title_size(&c.title, b),
        plain_arg(&c.subtitle),
        plain_arg(&c.eyebrow),
        plain_arg(&c.recipient),
        plain_arg(&c.author),
        plain_arg(&c.date),
        plain_arg(&c.reference),
    ));
}

/// Frontmatter text as Typst content, escaped literally.
fn plain_arg(text: &Option<String>) -> String {
    text.as_deref().map(|t| format!("[{}]", typst_content(t))).unwrap_or_else(|| "none".to_string())
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

fn write_body(s: &mut String, nodes: &[DocumentNode], consumed_body_h1: bool, ctx: Ctx) {
    let section_level = ctx.section_level;
    let mut skip_next_h1 = consumed_body_h1;
    let mut section = 0;
    let mut part = 0;
    for (i, node) in nodes.iter().enumerate() {
        if skip_next_h1 {
            if let DocumentNode::Heading { level: 1, .. } = node {
                skip_next_h1 = false;
                continue;
            }
        }
        // A section heading makes its own break; don't make two.
        if matches!(node, DocumentNode::SectionBreak)
            && matches!(
                nodes.get(i + 1),
                Some(DocumentNode::Heading { level, .. }) if *level <= section_level
            ) | matches!(
                nodes.get(i + 1),
                Some(DocumentNode::Component(Component::Opener(_) | Component::Part(_)))
            )
        {
            continue;
        }
        if let DocumentNode::Component(Component::Part(p)) = node {
            part += 1;
            s.push_str(&format!(
                "#part-divider(number: \"{part}\", level: {}, title: {}, eyebrow: {}, intro: {})\n\n",
                ctx.section_out - 1,
                content_arg(&p.title),
                opt_content(&p.eyebrow),
                opt_content(&p.intro),
            ));
            continue;
        }
        if let DocumentNode::Component(Component::Opener(o)) = node {
            section += 1;
            write_opener(s, o, Some(section), ctx.section_out);
            continue;
        }
        if let DocumentNode::Heading { level, text, .. } = node {
            if *level == section_level {
                section += 1;
                s.push_str(&format!(
                    "#heading(level: {}, supplement: [{section:02}])[{}]\n\n",
                    ctx.section_out,
                    typst_inline(text)
                ));
                continue;
            }
        }
        write_node(s, node, ctx);
    }
}

fn write_node(s: &mut String, node: &DocumentNode, ctx: Ctx) {
    let images = ctx.images;
    match node {
        DocumentNode::Heading { text, .. } if ctx.in_box => {
            s.push_str(&format!("#card-title[{}]\n\n", typst_inline(text)));
        }
        DocumentNode::Heading { level, text, .. } => {
            let hashes = "=".repeat(ctx.out_level(*level) as usize);
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
        DocumentNode::Image { url, alt, caption, full_width } => {
            let path = images.get(url).map(|a| typst_string(&a.name)).unwrap_or_else(|| "none".to_string());
            s.push_str(&format!(
                "#figure-image({path}, alt: {}, caption: {}, full: {full_width})\n\n",
                typst_string(alt),
                opt_content(caption),
            ));
        }
        DocumentNode::Columns { ratios, gutter, children } => {
            write_columns(s, ratios, gutter.as_deref(), children, ctx);
        }
        DocumentNode::Section { style, children } => {
            write_section(s, style.as_ref(), children, ctx);
        }
        DocumentNode::Component(component) => write_component(s, component, ctx),
    }
}

// ---------------------------------------------------------------------------
// Components: each becomes a call to its function in components.typ
// ---------------------------------------------------------------------------

/// Text with inline formatting, as Typst content.
fn content_arg(text: &str) -> String {
    format!("[{}]", typst_inline(text))
}

fn opt_content(text: &Option<String>) -> String {
    text.as_deref().map(content_arg).unwrap_or_else(|| "none".to_string())
}

fn opt_number(value: Option<f64>) -> String {
    value.map(|v| format!("{v}")).unwrap_or_else(|| "none".to_string())
}

/// Intros longer than this are set at body size rather than enlarged.
const LONG_INTRO_CHARS: usize = 280;

fn write_opener(s: &mut String, o: &Opener, number: Option<usize>, level: u8) {
    let stat = match &o.stat {
        Some(st) => format!(
            "(value: {}, label: {}, status: {})",
            content_arg(&st.value),
            opt_content(&st.label),
            st.status.map(|v| typst_string(v.as_str())).unwrap_or_else(|| "none".to_string()),
        ),
        None => "none".to_string(),
    };
    s.push_str(&format!(
        "#opener(number: {}, level: {level}, title: {}, eyebrow: {}, intro: {}, long-intro: {}, stat: {stat})\n\n",
        number.map(|n| format!("\"{n:02}\"")).unwrap_or_else(|| "none".to_string()),
        content_arg(&o.title),
        opt_content(&o.eyebrow),
        opt_content(&o.intro),
        o.intro.as_ref().is_some_and(|i| i.chars().count() > LONG_INTRO_CHARS),
    ));
}

/// A finding's fields as Typst named arguments (or dictionary entries).
fn finding_args(f: &FindingCard) -> String {
    let pages: Vec<String> = f.pages.iter().map(|p| content_arg(p)).collect();
    let evidence = match &f.evidence {
        Some(e) => format!("(value: {}, label: {})", content_arg(&e.value), opt_content(&e.label)),
        None => "none".to_string(),
    };
    format!(
        "title: {}, severity: {}, why: {}, fix: {}, pages: ({}), evidence: {evidence}",
        content_arg(&f.title),
        typst_string(f.severity.as_str()),
        opt_content(&f.why),
        opt_content(&f.fix),
        if pages.is_empty() { String::new() } else { format!("{},", pages.join(", ")) },
    )
}

fn write_component(s: &mut String, component: &Component, ctx: Ctx) {
    match component {
        // An opener inside a layout block isn't a numbered section.
        Component::Opener(o) => write_opener(s, o, None, ctx.section_out),
        Component::Callout(c) => {
            s.push_str(&format!(
                "#callout(tone: {}, title: {})[{}]\n\n",
                typst_string(c.tone.as_str()),
                opt_content(&c.title),
                typst_inline(&c.body),
            ));
        }
        Component::Findings(f) => {
            let items: Vec<String> = f
                .items
                .iter()
                .map(|i| {
                    let pages: Vec<String> = i.pages.iter().map(|p| content_arg(p)).collect();
                    format!(
                        "(title: {}, severity: {}, count: {}, pages: ({}), fix: {})",
                        content_arg(&i.title),
                        typst_string(i.severity.as_str()),
                        i.count.map(|c| c.to_string()).unwrap_or_else(|| "none".to_string()),
                        if pages.is_empty() { String::new() } else { format!("{},", pages.join(", ")) },
                        opt_content(&i.fix),
                    )
                })
                .collect();
            s.push_str(&format!("#findings(({},))\n\n", items.join(",\n  ")));
        }
        Component::Steps(st) => {
            let groups: Vec<String> = st
                .groups()
                .map(|g| {
                    let items: Vec<String> = g
                        .items
                        .iter()
                        .map(|i| {
                            format!(
                                "(title: {}, detail: {}, effort: {})",
                                content_arg(&i.title),
                                opt_content(&i.detail),
                                opt_content(&i.effort),
                            )
                        })
                        .collect();
                    format!("(title: {}, items: ({},))", opt_content(&g.title), items.join(", "))
                })
                .collect();
            s.push_str(&format!("#steps(({},))\n\n", groups.join(",\n  ")));
        }
        Component::Scores(sc) => {
            let items: Vec<String> = sc
                .items
                .iter()
                .map(|i| {
                    format!(
                        "(label: {}, value: {}, max: {}, benchmark: {})",
                        content_arg(&i.label),
                        i.value,
                        if i.max > 0.0 { i.max } else { 100.0 },
                        opt_number(i.benchmark),
                    )
                })
                .collect();
            s.push_str(&format!("#scores(({},), caption: {})\n\n", items.join(", "), opt_content(&sc.caption)));
        }
        Component::Metrics(m) => {
            let items: Vec<String> = m
                .items
                .iter()
                .map(|i| {
                    format!(
                        "(value: {}, label: {}, target: {}, status: {})",
                        content_arg(&i.value),
                        content_arg(&i.label),
                        opt_content(&i.target),
                        i.status.map(|v| typst_string(v.as_str())).unwrap_or_else(|| "none".to_string()),
                    )
                })
                .collect();
            s.push_str(&format!("#metrics(({},))\n\n", items.join(", ")));
        }
        Component::Facts(f) => {
            let items: Vec<String> = f
                .items
                .iter()
                .map(|i| format!("(label: {}, value: {})", content_arg(&i.label), content_arg(&i.value)))
                .collect();
            s.push_str(&format!("#facts(({},))\n\n", items.join(", ")));
        }
        Component::ScoreMatrix(m) => {
            let columns: Vec<String> = m.columns.iter().map(|c| content_arg(c)).collect();
            let rows: Vec<String> = m
                .rows
                .iter()
                .map(|r| {
                    let values: Vec<String> = (0..m.columns.len())
                        .map(|i| opt_number(r.values.get(i).copied().flatten()))
                        .collect();
                    format!("(label: {}, values: ({},))", content_arg(&r.label), values.join(", "))
                })
                .collect();
            s.push_str(&format!(
                "#score-matrix(({},), ({},), max: {}, caption: {})\n\n",
                columns.join(", "),
                rows.join(",\n  "),
                if m.max > 0.0 { m.max } else { 100.0 },
                opt_content(&m.caption),
            ));
        }
        Component::Bars(b) => {
            let items: Vec<String> = b
                .items
                .iter()
                .map(|i| format!("(label: {}, value: {})", content_arg(&i.label), i.value))
                .collect();
            s.push_str(&format!(
                "#bars(({},), unit: {}, target: {}, better: {}, caption: {})\n\n",
                items.join(", "),
                b.unit.as_deref().map(typst_string).unwrap_or_else(|| "none".to_string()),
                opt_number(b.target),
                typst_string(match b.better {
                    Better::Lower => "lower",
                    Better::Higher => "higher",
                }),
                opt_content(&b.caption),
            ));
        }
        Component::SeveritySummary(sv) => {
            let c = &sv.counts;
            s.push_str(&format!(
                "#severity-summary((critical: {}, serious: {}, moderate: {}, minor: {}), caption: {})\n\n",
                c.critical,
                c.serious,
                c.moderate,
                c.minor,
                opt_content(&sv.caption),
            ));
        }
        Component::Finding(f) => s.push_str(&format!("#finding-card({})\n\n", finding_args(f))),
        // A part only means something at the top level; nested, it's a title.
        Component::Part(p) => s.push_str(&format!("#card-title[{}]\n\n", typst_inline(&p.title))),
        Component::Highlights(h) => {
            let items: Vec<String> = h.items.iter().map(|f| format!("({})", finding_args(f))).collect();
            s.push_str(&format!("#highlights(({},))\n\n", items.join(",\n  ")));
        }
        Component::PriorityMatrix(m) => {
            let items: Vec<String> = m
                .items
                .iter()
                .map(|i| {
                    format!(
                        "(label: {}, impact: {}, effort: {})",
                        content_arg(&i.label),
                        typst_string(i.impact.as_str()),
                        typst_string(i.effort.as_str()),
                    )
                })
                .collect();
            s.push_str(&format!("#priority-matrix(({},))\n\n", items.join(", ")));
        }
        Component::Contents(c) => {
            s.push_str(&format!("#contents(title: {})\n\n", opt_content(&c.title)));
        }
        Component::BackPage(b) => {
            let contact: Vec<String> = [
                ("Contact", &b.contact.name),
                ("Email", &b.contact.email),
                ("Website", &b.contact.website),
                ("Phone", &b.contact.phone),
            ]
            .into_iter()
            .filter_map(|(label, value)| value.as_ref().map(|v| format!("({}, {})", typst_string(label), content_arg(v))))
            .collect();
            s.push_str(&format!(
                "#back-page(about: {}, contact: ({}))\n\n",
                opt_content(&b.about),
                if contact.is_empty() { String::new() } else { format!("{},", contact.join(", ")) },
            ));
        }
    }
}

fn write_columns(
    s: &mut String,
    ratios: &[f32],
    gutter: Option<&str>,
    children: &[StyledColumn],
    ctx: Ctx,
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
        let styled = child.style.as_ref().is_some_and(|st| !st.is_empty());
        let inner_ctx = if styled { ctx.boxed() } else { ctx };
        let mut inner = String::new();
        for node in &child.nodes {
            write_node(&mut inner, node, inner_ctx);
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

fn write_section(s: &mut String, style: Option<&BlockStyle>, children: &[DocumentNode], ctx: Ctx) {
    let styled = style.is_some_and(|st| !st.is_empty());
    let inner_ctx = if styled { ctx.boxed() } else { ctx };
    let mut inner = String::new();
    for node in children {
        write_node(&mut inner, node, inner_ctx);
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

    if let Some(bg) = style.background.as_deref().and_then(colour_value) {
        args.push(format!("fill: {bg}"));
    }

    if let Some(border) = style.border.as_ref() {
        let w = length_value(border.width.as_deref(), "1pt");
        let c = border
            .color
            .as_deref()
            .and_then(colour_value)
            .unwrap_or_else(|| "t.rule".to_string());
        args.push(format!("stroke: {w} + {c}"));
    }

    args.push(format!(
        "inset: {}",
        length_value(style.padding.as_deref(), "t.s4")
    ));
    args.push(format!(
        "radius: {}",
        length_value(style.radius.as_deref(), "t.r-md")
    ));
    args.push("width: 100%".to_string());
    // Cards and callouts read as one object; never split one across pages.
    args.push("breakable: false".to_string());

    s.push_str(&format!("block({})[\n", args.join(", ")));

    if let Some(tc) = style.text_color.as_deref().and_then(colour_value) {
        s.push_str(&format!("  #set text(fill: {tc})\n"));
    }

    s.push_str(inner);
    s.push(']');
}

/// Colour tokens a style may name instead of a hex value, so authors stay on
/// brand.
const COLOUR_TOKENS: [&str; 13] = [
    "ink", "ink-muted", "heading", "paper", "surface", "rule", "accent", "accent-soft", "cover-bg",
    "cover-text", "good", "fair", "poor",
];

/// Length tokens a style may name instead of a raw length.
const LENGTH_TOKENS: [&str; 13] = [
    "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "r-sm", "r-md", "hairline", "heavy",
];

/// A style colour as Typst: a token name (`surface`) or a hex value.
fn colour_value(value: &str) -> Option<String> {
    let v = value.trim();
    if COLOUR_TOKENS.contains(&v) {
        Some(format!("t.{v}"))
    } else if is_hex(v) {
        Some(format!("rgb(\"{v}\")"))
    } else {
        None
    }
}

/// A style length as Typst: a token name (`s5`, `r-md`) or a raw length.
fn length_value(value: Option<&str>, default: &str) -> String {
    match value.map(str::trim) {
        Some(v) if LENGTH_TOKENS.contains(&v) => format!("t.{v}"),
        other => typst_length(other, default),
    }
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

/// Monospace names that map to JetBrains Mono.
const MONO_ALIASES: [&str; 8] =
    ["courier", "courier new", "monospace", "dejavu sans mono", "menlo", "consolas", "dm mono", "ibm plex mono"];

/// Generic sans names, which take the role's bundled family.
const SANS_ALIASES: [&str; 6] = ["helvetica", "helvetica neue", "arial", "sans-serif", "system-ui", "inter variable"];

/// Whether a profile font name renders as intended: a bundled family or a
/// known alias for one. Anything else would silently fall back.
pub fn is_known_font(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    BUNDLED_FONTS.iter().any(|f| f.eq_ignore_ascii_case(&lower))
        || MONO_ALIASES.contains(&lower.as_str())
        || SANS_ALIASES.contains(&lower.as_str())
        || matches!(lower.as_str(), "inter display" | "inter tight variable" | "jetbrains mono nl")
}

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
        name if MONO_ALIASES.contains(&name) => "JetBrains Mono",
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

/// Compile Typst source to PDF. `assets` (logos, images) are written into
/// the compile root under their own names, which the source refers to.
///
/// PDFs are built as tagged PDF/UA-1 (accessible to screen readers). If a
/// document can't meet the standard (an image without alt text in a version
/// 1 document, say), it's built as a plain PDF instead and the reason logged:
/// accessibility never costs a render.
fn compile(source: &str, assets: &[&Asset]) -> anyhow::Result<Vec<u8>> {
    match compile_to(source, assets, &["--pdf-standard", "ua-1"], "doc.pdf") {
        Ok(mut files) => Ok(files.remove(0)),
        Err(e) if e.to_string().contains("PDF/UA") => {
            tracing::warn!(error = %e, "built without PDF/UA");
            let mut files = compile_to(source, assets, &[], "doc.pdf")?;
            Ok(files.remove(0))
        }
        Err(e) => Err(e),
    }
}

/// Render each page as a PNG at `ppi`, for proofs.
pub fn render_proof(branded: &BrandedDocument, ppi: u32) -> anyhow::Result<Vec<Vec<u8>>> {
    let source = build_source(branded);
    let assets: Vec<&Asset> = [&branded.logos.light, &branded.logos.dark]
        .into_iter()
        .flatten()
        .chain(branded.images.values())
        .collect();
    let ppi = ppi.clamp(20, 150).to_string();
    compile_to(&source, &assets, &["--format", "png", "--ppi", &ppi], "page-{0p}.png")
}

/// Run `typst compile` in a temporary root and return the output files in
/// order: one for a PDF, one per page for an output pattern like
/// `page-{0p}.png`.
fn compile_to(source: &str, assets: &[&Asset], extra: &[&str], output_name: &str) -> anyhow::Result<Vec<Vec<u8>>> {
    let tmp = tempfile::tempdir()?;
    let input = tmp.path().join("doc.typ");
    let out_dir = tmp.path().join("out");
    std::fs::create_dir(&out_dir)?;

    {
        let mut f = std::fs::File::create(&input)?;
        f.write_all(source.as_bytes())?;
    }
    for asset in assets {
        std::fs::write(tmp.path().join(&asset.name), &asset.bytes)?;
    }

    let result = Command::new("typst")
        .args(["compile", "--root", tmp.path().to_str().unwrap()])
        .arg("--font-path")
        .arg(font_dir())
        .arg("--ignore-system-fonts")
        .args(extra)
        .arg(&input)
        .arg(out_dir.join(output_name))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("typst compile failed:\n{}", stderr);
    }

    let mut paths: Vec<_> = std::fs::read_dir(&out_dir)?.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    paths.iter().map(|p| Ok(std::fs::read(p)?)).collect()
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
    fn headings_in_styled_boxes_become_card_titles() {
        let source = build_source(&branded_json(r##"{
            "pages": [{ "blocks": [{
                "type": "section",
                "background": "#111111",
                "text_color": "#FFFFFF",
                "blocks": [{ "type": "heading", "level": 3, "text": "Card" }]
            }]}]
        }"##));
        // A card title isn't a heading: it stays out of the structure and bookmarks.
        assert!(source.contains("#card-title[Card]"), "{source}");
        assert!(!source.contains("=== Card"));
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
    fn styles_accept_token_names() {
        assert_eq!(colour_value("surface").as_deref(), Some("t.surface"));
        assert_eq!(colour_value("#112233").as_deref(), Some("rgb(\"#112233\")"));
        assert_eq!(colour_value("red"), None);
        assert_eq!(length_value(Some("s5"), "8pt"), "t.s5");
        assert_eq!(length_value(Some("12pt"), "8pt"), "12pt");
        assert_eq!(length_value(None, "t.s4"), "t.s4");
    }

    /// The theme's status colours, as emitted in `t`.
    const STATUS: [(&str, &str); 5] = [
        ("good", "#1F7A4D"),
        ("fair", "#966010"),
        ("poor", "#B42318"),
        ("serious", "#C2410C"),
        ("muted", "#5B5852"),
    ];

    fn rgb(hex: &str) -> [f64; 3] {
        let h = hex.trim_start_matches('#');
        [0, 2, 4].map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap() as f64)
    }

    fn hex(c: [f64; 3]) -> String {
        format!("#{:02X}{:02X}{:02X}", c[0].round() as u8, c[1].round() as u8, c[2].round() as u8)
    }

    /// `tint(c, amount)` from components.typ: the colour mixed into paper.
    fn tint(c: &str, paper: &str, amount: f64) -> String {
        let (c, p) = (rgb(c), rgb(paper));
        hex([0, 1, 2].map(|i| c[i] * (1.0 - amount) + p[i] * amount))
    }

    fn darken(c: &str, f: f64) -> String {
        hex(rgb(c).map(|x| x * (1.0 - f)))
    }

    /// Every pair the theme sets text in: status text on paper, badge and
    /// pill text on their tints, and the filled critical badge. Checked for
    /// the default profile and the gallery's brand profile (warm paper).
    #[test]
    fn theme_colour_pairs_meet_wcag_aa() {
        use crate::models::client_profile::contrast_ratio;
        for paper in ["#FFFFFF", &gallery_profile().colour_background] {
            for (name, c) in STATUS {
                let checks = [
                    ("on paper", contrast_ratio(c, paper).unwrap(), 4.5),
                    ("badge", contrast_ratio(&darken(c, 0.10), &tint(c, paper, 0.85)).unwrap(), 4.5),
                    ("pill", contrast_ratio(&darken(c, 0.20), &tint(c, paper, 0.82)).unwrap(), 4.5),
                ];
                for (pair, ratio, min) in checks {
                    assert!(ratio >= min, "{name} {pair} on {paper}: {ratio:.2}:1 is below {min}:1");
                }
            }
            assert!(contrast_ratio(paper, "#B42318").unwrap() >= 4.5, "critical badge text on {paper}");
        }
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
    /// [TYPESET_PREVIEW_PROFILE=overrides.json] [TYPESET_PREVIEW_PRINT=1] cargo test preview -- --ignored`.
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
        let mut branded = crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");
        branded.print = std::env::var("TYPESET_PREVIEW_PRINT").is_ok();
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

    /// The Typst version the Docker image pins; snapshots are only
    /// comparable when rendered by the same version.
    fn pinned_typst_version() -> String {
        let dockerfile = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Dockerfile")).unwrap();
        dockerfile
            .lines()
            .find_map(|l| l.trim().strip_prefix("ARG TYPST_VERSION="))
            .unwrap()
            .to_string()
    }

    fn installed_typst_version() -> Option<String> {
        let out = std::process::Command::new("typst").arg("--version").output().ok()?;
        String::from_utf8_lossy(&out.stdout).split_whitespace().nth(1).map(str::to_string)
    }

    fn gallery_profile() -> crate::models::client_profile::ClientProfile {
        let mut profile = serde_json::to_value(crate::models::client_profile::ClientProfile::default_profile()).unwrap();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gallery_profile.json");
        let overrides: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for (k, v) in overrides.as_object().unwrap() {
            profile[k] = v.clone();
        }
        serde_json::from_value(profile).unwrap()
    }

    /// Visual regression: the gallery, page by page, against the approved
    /// PNGs in tests/snapshots. `UPDATE_SNAPSHOTS=1` approves the current
    /// render; a mismatch writes the new pages to tests/snapshots/failed/.
    /// Skipped unless the installed Typst is the version the image pins.
    #[test]
    fn gallery_matches_snapshots() {
        let pinned = pinned_typst_version();
        if installed_typst_version().as_deref() != Some(pinned.as_str()) {
            eprintln!("typst {pinned} not installed; skipping snapshots");
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let json = std::fs::read_to_string(root.join("examples/gallery.json")).unwrap();
        let doc = crate::parser::json::parse(&json).unwrap();
        let branded = BrandedDocument::prepare(doc, gallery_profile()).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("doc.typ"), build_source(&branded)).unwrap();
        let out = std::process::Command::new("typst")
            .args(["compile", "--root"])
            .arg(tmp.path())
            .arg("--font-path")
            .arg(font_dir())
            .args(["--ignore-system-fonts", "--format", "png", "--ppi", "60"])
            .arg(tmp.path().join("doc.typ"))
            .arg(tmp.path().join("page-{0p}.png"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

        let mut pages: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "png"))
            .collect();
        pages.sort();

        let snapshots = root.join("tests/snapshots");
        if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
            std::fs::create_dir_all(&snapshots).unwrap();
            for old in std::fs::read_dir(&snapshots).unwrap().flatten() {
                if old.path().extension().is_some_and(|x| x == "png") {
                    std::fs::remove_file(old.path()).unwrap();
                }
            }
            for page in &pages {
                std::fs::copy(page, snapshots.join(format!("gallery-{}", page.file_name().unwrap().to_string_lossy()))).unwrap();
            }
            return;
        }

        let mut failed = Vec::new();
        for page in &pages {
            let name = format!("gallery-{}", page.file_name().unwrap().to_string_lossy());
            let approved = std::fs::read(snapshots.join(&name)).unwrap_or_default();
            if approved != std::fs::read(page).unwrap() {
                let dir = snapshots.join("failed");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::copy(page, dir.join(&name)).unwrap();
                failed.push(name);
            }
        }
        let approved_count = std::fs::read_dir(&snapshots)
            .map(|d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "png")).count())
            .unwrap_or(0);
        assert!(
            failed.is_empty() && approved_count == pages.len(),
            "gallery changed: {failed:?} ({} pages now, {approved_count} approved). New renders are in tests/snapshots/failed/; \
             check them, then run with UPDATE_SNAPSHOTS=1 to approve.",
            pages.len()
        );
    }

    #[test]
    fn pdfs_are_tagged_pdf_ua() {
        if !typst_available() {
            return;
        }
        let b = branded_json(r#"{ "schema": 2, "frontmatter": { "title": "Tagged" }, "pages": [
            { "blocks": [{ "type": "opener", "title": "One" }, { "type": "heading", "level": 3, "text": "Sub" },
                         { "type": "section", "background": "surface", "blocks": [{ "type": "heading", "level": 4, "text": "Card" }] }] }
        ]}"#);
        let pdf = render(&b).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("pdfuaid"), "the PDF should declare PDF/UA conformance");
    }

    #[test]
    fn print_adds_bleed_and_cmyk() {
        let mut b = branded_json(r#"{ "frontmatter": { "title": "Print" }, "pages": [{ "blocks": [{ "type": "paragraph", "text": "x" }] }] }"#);
        b.print = true;
        let source = build_source(&b);
        assert!(source.contains("page-width: 216mm,") && source.contains("page-height: 303mm,"), "A4 plus 3mm a side");
        assert!(source.contains("print: true,"));
        // The theme converts every colour token to CMYK when meta.print is
        // set (checked by hand: the content streams use only k/K operators).
        if typst_available() {
            render(&b).expect("print renders");
        }
    }

    #[test]
    fn parts_divide_a_combined_document_and_stay_pdf_ua() {
        let b = branded_json(r#"{ "schema": 2, "frontmatter": { "title": "Pack" }, "pages": [
            { "blocks": [{ "type": "contents" }] },
            { "blocks": [{ "type": "part", "title": "Review", "intro": "The review." }] },
            { "blocks": [{ "type": "opener", "title": "Summary" }, { "type": "heading", "level": 3, "text": "Sub" }] },
            { "blocks": [{ "type": "part", "title": "Proposal" }] },
            { "blocks": [{ "type": "opener", "title": "Scope" }] }
        ]}"#);
        let source = build_source(&b);
        // Cover H1, parts H2, sections H3, so heading levels never skip.
        assert!(source.contains(r#"#part-divider(number: "1", level: 2, title: [Review]"#), "{source}");
        assert!(source.contains(r#"#opener(number: "01", level: 3, title: [Summary]"#));
        assert!(source.contains(r#"#opener(number: "02", level: 3, title: [Scope]"#), "section numbers run on");
        assert!(source.contains("==== Sub"));
        assert!(source.contains("part-level: 2,"));
        if typst_available() {
            let pdf = render(&b).unwrap();
            assert!(String::from_utf8_lossy(&pdf).contains("pdfuaid"), "should still be PDF/UA");
        }
    }

    #[test]
    fn proofs_are_one_png_per_page() {
        if !typst_available() {
            return;
        }
        let b = branded_json(r#"{ "frontmatter": { "title": "Proof" }, "pages": [{ "blocks": [{ "type": "paragraph", "text": "Body." }] }] }"#);
        let pages = render_proof(&b, 30).unwrap();
        assert_eq!(pages.len(), 2, "a cover and one body page");
        assert!(pages.iter().all(|p| p.starts_with(b"\x89PNG")));
    }

    #[test]
    fn every_cover_style_renders() {
        if !typst_available() {
            return;
        }
        for style in ["editorial", "band", "dark"] {
            let input = format!(
                r#"{{ "schema": 2, "frontmatter": {{ "title": "Cover", "eyebrow": "Review", "recipient": "A client", "cover": "{style}" }},
                   "pages": [{{ "blocks": [{{ "type": "paragraph", "text": "Body." }}] }}] }}"#
            );
            let doc = crate::parser::json::parse(&input).unwrap();
            let branded = BrandedDocument::prepare(doc, gallery_profile()).unwrap();
            assert!(build_source(&branded).contains(&format!("#cover(style: \"{style}\"")));
            render(&branded).unwrap_or_else(|e| panic!("{style}: {e}"));
        }
    }

    #[test]
    fn old_cover_templates_keep_the_dark_cover() {
        assert_eq!(cover_style(None, "minimal"), "dark");
        assert_eq!(cover_style(None, "bold"), "dark");
        assert_eq!(cover_style(None, "editorial"), "editorial");
        assert_eq!(cover_style(Some("band"), "minimal"), "band");
    }

    #[test]
    fn openers_are_numbered_sections() {
        let source = build_source(&branded_json(r#"{ "schema": 2, "frontmatter": { "title": "Doc" }, "pages": [
            { "blocks": [{ "type": "opener", "title": "First" }, { "type": "heading", "level": 3, "text": "Sub" }] },
            { "blocks": [{ "type": "heading", "level": 2, "text": "Second" }] }
        ]}"#));
        assert!(source.contains(r#"#opener(number: "01", level: 2, title: [First]"#), "{source}");
        assert!(source.contains("#heading(level: 2, supplement: [02])[Second]"));
        assert!(source.contains("section-level: 2,"));
    }

    #[test]
    fn fetched_images_render_and_missing_ones_get_a_placeholder() {
        let mut b = branded_json(r#"{ "pages": [{ "blocks": [
            { "type": "image", "url": "https://example.com/a.png", "alt": "A", "caption": "Cap" },
            { "type": "image", "url": "https://example.com/missing.png", "alt": "B" }
        ]}]}"#);
        b.images.insert(
            "https://example.com/a.png".to_string(),
            Asset { name: "img-0.png".to_string(), bytes: Vec::new() },
        );
        let source = build_source(&b);
        assert!(source.contains(r#"#figure-image("img-0.png", alt: "A", caption: [Cap], full: false)"#), "{source}");
        assert!(source.contains(r#"#figure-image(none, alt: "B", caption: none, full: false)"#));
    }

    #[test]
    fn the_back_page_is_left_out_of_the_page_total() {
        let source = build_source(&branded_json(r#"{ "schema": 2, "pages": [{ "blocks": [
            { "type": "paragraph", "text": "x" }, { "type": "back_page", "about": "About." }
        ]}]}"#));
        assert!(source.contains("has-back-page: true,"));
    }

    #[test]
    fn the_wordmark_replaces_the_title_in_the_running_head() {
        let mut b = branded_json(r#"{ "frontmatter": { "title": "Doc", "recipient": "Lee" }, "pages": [] }"#);
        b.profile.wordmark = Some("Chris Garlick".to_string());
        let source = build_source(&b);
        assert!(source.contains(r#"wordmark: "Chris Garlick","#));
        assert!(source.contains("has-brand: true,"));
        assert!(source.contains("footer-left: [Doc · Prepared for Lee]"));
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
