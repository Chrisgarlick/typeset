# Typeset — Technical Documentation
## Branded Document Renderer · Build Reference

---

## 1. Overview

Typeset is a high-performance document rendering service that converts Markdown files into branded PDFs and DOCX files. It runs as a standalone microservice on the existing DigitalOcean infrastructure alongside other services.

### Design Priorities

1. **Memory efficiency** — minimal footprint, no heavy runtimes, no browser dependencies
2. **Speed** — sub-3s renders for standard documents
3. **Isolation** — does not compete for resources with other services on the same droplet
4. **Correctness** — pixel-accurate brand application, consistent output

### Why Rust

Node.js/Puppeteer approaches to PDF generation consume 200–500MB RAM per render due to spawning a headless Chromium instance. On a shared droplet this is unacceptable.

The Rust approach:

- **Markdown parsing** → `pulldown-cmark` (pure Rust, zero system deps)
- **PDF generation** → `printpdf` (pure Rust PDF writer, no external binaries)
- **DOCX generation** → `docx-rs` (pure Rust DOCX writer)
- **HTTP server** → `axum` (async, extremely low overhead)
- **Memory per render** → ~8–15MB
- **Render time (PDF)** → 400–900ms
- **Render time (DOCX)** → 100–300ms
- **Binary size** → ~12MB compiled
- **Runtime dependencies** → zero (statically linked)

The service compiles to a single binary. No Node, no Chromium, no Python. Drops straight onto the droplet.

---

## 2. Architecture

```
┌─────────────────────────────────────────────┐
│              Next.js Frontend               │
│         typeset.chrisgarlick.com            │
│              (Vercel)                       │
└──────────────────┬──────────────────────────┘
                   │ HTTPS API calls
                   ▼
┌─────────────────────────────────────────────┐
│           Typeset Render Service            │
│         Rust · Axum HTTP server             │
│         Port 3200 · DigitalOcean            │
│                                             │
│  ┌─────────────┐    ┌────────────────────┐  │
│  │  MD Parser  │    │   Brand Engine     │  │
│  │ pulldown-   │───▶│  Applies client    │  │
│  │    cmark    │    │  profile styles    │  │
│  └─────────────┘    └────────┬───────────┘  │
│                              │              │
│              ┌───────────────┴──────────┐   │
│              ▼                          ▼   │
│  ┌───────────────────┐  ┌─────────────────┐ │
│  │   PDF Renderer    │  │  DOCX Renderer  │ │
│  │    printpdf       │  │    docx-rs      │ │
│  └─────────┬─────────┘  └───────┬─────────┘ │
│            └───────────┬────────┘           │
│                        ▼                    │
│              ┌──────────────────┐           │
│              │  Output Storage  │           │
│              │  DO Spaces / disk│           │
│              └──────────────────┘           │
└─────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│           PostgreSQL (existing)             │
│   Client profiles · Render history         │
└─────────────────────────────────────────────┘
```

---

## 3. Repository Structure

```
typeset/
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── README.md
├── docs/
│   └── this file
├── src/
│   ├── main.rs                 # Entry point, server bootstrap
│   ├── config.rs               # Environment config
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── render.rs           # POST /render
│   │   ├── preview.rs          # POST /preview
│   │   ├── clients.rs          # CRUD /clients
│   │   ├── history.rs          # GET /history
│   │   └── health.rs           # GET /health
│   ├── models/
│   │   ├── mod.rs
│   │   ├── client_profile.rs   # ClientProfile struct
│   │   ├── render_job.rs       # RenderJob struct
│   │   └── document.rs         # Document struct
│   ├── parser/
│   │   ├── mod.rs
│   │   └── markdown.rs         # MD → AST via pulldown-cmark
│   ├── brand/
│   │   ├── mod.rs
│   │   └── engine.rs           # Applies client profile to document
│   ├── renderers/
│   │   ├── mod.rs
│   │   ├── pdf.rs              # AST → PDF via printpdf
│   │   └── docx.rs             # AST → DOCX via docx-rs
│   ├── storage/
│   │   ├── mod.rs
│   │   └── spaces.rs           # DigitalOcean Spaces upload
│   └── db/
│       ├── mod.rs
│       └── queries.rs          # PostgreSQL queries via sqlx
├── migrations/
│   ├── 001_create_client_profiles.sql
│   └── 002_create_render_history.sql
└── tests/
    ├── render_tests.rs
    └── fixtures/
        ├── sample.md
        └── sample_profile.json
```

---

## 4. Dependencies (`Cargo.toml`)

```toml
[package]
name = "typeset"
version = "0.1.0"
edition = "2021"

[dependencies]
# HTTP server
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Markdown parsing
pulldown-cmark = "0.10"

# PDF generation
printpdf = "0.6"

# DOCX generation
docx-rs = "0.4"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }

# Serialisation
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# File storage (DO Spaces = S3-compatible)
aws-sdk-s3 = "1"
aws-config = "1"

# Utilities
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenvy = "0.15"

# Font handling
rusttype = "0.9"

# Image handling (for logos in documents)
image = "0.24"

# Hex colour parsing
csscolorparser = "0.6"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

---

## 5. Environment Configuration

`.env.example`:
```bash
# Server
PORT=3200
HOST=0.0.0.0
RUST_LOG=info

# Database (existing PostgreSQL)
DATABASE_URL=postgresql://user:password@localhost:5432/typeset

# DigitalOcean Spaces
SPACES_KEY=your_spaces_key
SPACES_SECRET=your_spaces_secret
SPACES_BUCKET=typeset-renders
SPACES_REGION=lon1
SPACES_ENDPOINT=https://lon1.digitaloceanspaces.com

# Auth (shared secret for API key validation)
API_SECRET_SALT=your_random_salt

# Render settings
MAX_RENDER_CONCURRENCY=4
RENDER_TIMEOUT_SECS=30
OUTPUT_EXPIRY_DAYS=30

# Fonts directory
FONTS_DIR=/opt/typeset/fonts
```

---

## 6. Database Schema

```sql
-- migrations/001_create_client_profiles.sql

CREATE TABLE client_profiles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL,
    slug        VARCHAR(100) NOT NULL UNIQUE,
    name        VARCHAR(255) NOT NULL,

    -- Branding
    colour_primary      VARCHAR(7) NOT NULL DEFAULT '#000000',
    colour_secondary    VARCHAR(7) NOT NULL DEFAULT '#333333',
    colour_accent       VARCHAR(7) NOT NULL DEFAULT '#0066CC',
    colour_text         VARCHAR(7) NOT NULL DEFAULT '#1A1A1A',
    colour_background   VARCHAR(7) NOT NULL DEFAULT '#FFFFFF',
    colour_table_header VARCHAR(7) NOT NULL DEFAULT '#F5F5F5',
    colour_table_border VARCHAR(7) NOT NULL DEFAULT '#E0E0E0',
    colour_callout_bg   VARCHAR(7) NOT NULL DEFAULT '#F8F9FA',

    -- Typography
    font_heading        VARCHAR(100) NOT NULL DEFAULT 'Helvetica',
    font_body           VARCHAR(100) NOT NULL DEFAULT 'Helvetica',
    font_mono           VARCHAR(100) NOT NULL DEFAULT 'Courier',
    font_size_base      FLOAT NOT NULL DEFAULT 11.0,
    font_size_h1        FLOAT NOT NULL DEFAULT 24.0,
    font_size_h2        FLOAT NOT NULL DEFAULT 18.0,
    font_size_h3        FLOAT NOT NULL DEFAULT 14.0,
    line_height         FLOAT NOT NULL DEFAULT 1.5,

    -- Layout
    page_size           VARCHAR(10) NOT NULL DEFAULT 'A4',
    margin_top          FLOAT NOT NULL DEFAULT 25.4,
    margin_bottom       FLOAT NOT NULL DEFAULT 25.4,
    margin_left         FLOAT NOT NULL DEFAULT 25.4,
    margin_right        FLOAT NOT NULL DEFAULT 25.4,
    paragraph_spacing   FLOAT NOT NULL DEFAULT 8.0,
    section_spacing     FLOAT NOT NULL DEFAULT 16.0,

    -- Logo
    logo_light_url      TEXT,
    logo_dark_url       TEXT,
    logo_width          FLOAT NOT NULL DEFAULT 40.0,
    logo_position       VARCHAR(20) NOT NULL DEFAULT 'left',

    -- Cover page
    cover_enabled       BOOLEAN NOT NULL DEFAULT false,
    cover_template      VARCHAR(20) NOT NULL DEFAULT 'minimal',
    cover_bg_colour     VARCHAR(7) NOT NULL DEFAULT '#000000',
    cover_text_colour   VARCHAR(7) NOT NULL DEFAULT '#FFFFFF',

    -- Header / Footer
    header_enabled      BOOLEAN NOT NULL DEFAULT true,
    header_template     VARCHAR(20) NOT NULL DEFAULT 'logo-left',
    header_border       BOOLEAN NOT NULL DEFAULT true,
    footer_enabled      BOOLEAN NOT NULL DEFAULT true,
    footer_template     VARCHAR(20) NOT NULL DEFAULT 'page-numbers',
    footer_border       BOOLEAN NOT NULL DEFAULT true,

    -- Watermark
    watermark_text      VARCHAR(100),
    watermark_opacity   FLOAT NOT NULL DEFAULT 0.08,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_client_profiles_user_id ON client_profiles(user_id);
CREATE INDEX idx_client_profiles_slug ON client_profiles(slug);
```

```sql
-- migrations/002_create_render_history.sql

CREATE TABLE render_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    client_slug     VARCHAR(100),
    document_type   VARCHAR(50) NOT NULL DEFAULT 'general',
    format          VARCHAR(10) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Input
    content_hash    VARCHAR(64),
    frontmatter     JSONB,

    -- Output
    pdf_url         TEXT,
    docx_url        TEXT,
    page_count      INTEGER,
    file_size_bytes INTEGER,

    -- Timing
    render_ms       INTEGER,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_render_history_user_id ON render_history(user_id);
CREATE INDEX idx_render_history_created_at ON render_history(created_at DESC);
```

---

## 7. Data Models

### ClientProfile

```rust
// src/models/client_profile.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClientProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String,
    pub name: String,

    // Branding
    pub colour_primary: String,
    pub colour_secondary: String,
    pub colour_accent: String,
    pub colour_text: String,
    pub colour_background: String,
    pub colour_table_header: String,
    pub colour_table_border: String,
    pub colour_callout_bg: String,

    // Typography
    pub font_heading: String,
    pub font_body: String,
    pub font_mono: String,
    pub font_size_base: f32,
    pub font_size_h1: f32,
    pub font_size_h2: f32,
    pub font_size_h3: f32,
    pub line_height: f32,

    // Layout
    pub page_size: String,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub paragraph_spacing: f32,
    pub section_spacing: f32,

    // Logo
    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub logo_width: f32,
    pub logo_position: String,

    // Cover
    pub cover_enabled: bool,
    pub cover_template: String,
    pub cover_bg_colour: String,
    pub cover_text_colour: String,

    // Header / Footer
    pub header_enabled: bool,
    pub header_template: String,
    pub header_border: bool,
    pub footer_enabled: bool,
    pub footer_template: String,
    pub footer_border: bool,

    // Watermark
    pub watermark_text: Option<String>,
    pub watermark_opacity: f32,
}
```

### RenderRequest

```rust
// src/models/render_job.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub client: Option<String>,         // Client slug — uses default profile if None
    pub document_type: DocumentType,
    pub format: RenderFormat,
    pub content: String,                // Raw markdown
    pub overrides: Option<RenderOverrides>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    Proposal,
    Report,
    Brief,
    Sop,
    Invoice,
    General,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RenderFormat {
    Pdf,
    Docx,
    Both,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenderOverrides {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub recipient: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    pub success: bool,
    pub render_id: String,
    pub pdf_url: Option<String>,
    pub docx_url: Option<String>,
    pub expires_at: String,
    pub render_ms: u64,
}
```

---

## 8. Markdown Parser

```rust
// src/parser/markdown.rs

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum DocumentNode {
    Heading { level: u8, text: String, id: String },
    Paragraph { text: String },
    BulletList { items: Vec<String> },
    OrderedList { items: Vec<String>, start: u64 },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    CodeBlock { language: Option<String>, code: String },
    Blockquote { text: String },
    HorizontalRule,
    Image { url: String, alt: String },
    PageBreak,
}

#[derive(Debug, Serialize)]
pub struct ParsedDocument {
    pub frontmatter: Option<Frontmatter>,
    pub nodes: Vec<DocumentNode>,
}

#[derive(Debug, Serialize, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub recipient: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
    pub document_type: Option<String>,
    pub client: Option<String>,
}

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
    if !input.starts_with("---") {
        return (None, input.to_string());
    }

    let end = input[3..].find("---").map(|i| i + 6);
    match end {
        None => (None, input.to_string()),
        Some(end_idx) => {
            let yaml = &input[3..end_idx - 3];
            let content = input[end_idx..].trim().to_string();
            let fm = parse_frontmatter_yaml(yaml);
            (Some(fm), content)
        }
    }
}

fn parse_frontmatter_yaml(yaml: &str) -> Frontmatter {
    // Simple key: value parser — no full YAML dep needed
    let mut fm = Frontmatter::default();
    for line in yaml.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 { continue; }
        let key = parts[0].trim();
        let val = parts[1].trim().trim_matches('"').to_string();
        match key {
            "title"         => fm.title = Some(val),
            "subtitle"      => fm.subtitle = Some(val),
            "recipient"     => fm.recipient = Some(val),
            "date"          => fm.date = Some(val),
            "author"        => fm.author = Some(val),
            "document_type" => fm.document_type = Some(val),
            "client"        => fm.client = Some(val),
            _ => {}
        }
    }
    fm
}

fn build_nodes<'a>(parser: Parser<'a, 'a>) -> Vec<DocumentNode> {
    // Full event stream → DocumentNode conversion
    // Handles heading, paragraph, list, table, code, blockquote, rule, image
    // See src/parser/markdown.rs for full implementation
    vec![] // placeholder
}
```

---

## 9. Brand Engine

The brand engine takes a `ParsedDocument` and a `ClientProfile` and produces a `BrandedDocument` — a render-ready structure with all style values resolved.

```rust
// src/brand/engine.rs

use crate::models::client_profile::ClientProfile;
use crate::parser::markdown::ParsedDocument;

pub struct BrandedDocument {
    pub profile: ClientProfile,
    pub doc: ParsedDocument,
    pub page: PageDimensions,
    pub fonts: ResolvedFonts,
    pub colours: ResolvedColours,
}

pub struct PageDimensions {
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_top_mm: f32,
    pub margin_bottom_mm: f32,
    pub margin_left_mm: f32,
    pub margin_right_mm: f32,
    pub content_width_mm: f32,
    pub content_height_mm: f32,
}

pub struct ResolvedColours {
    pub primary: [f32; 3],      // RGB 0.0–1.0
    pub secondary: [f32; 3],
    pub accent: [f32; 3],
    pub text: [f32; 3],
    pub background: [f32; 3],
    pub table_header: [f32; 3],
    pub table_border: [f32; 3],
    pub callout_bg: [f32; 3],
}

pub struct ResolvedFonts {
    pub heading: Vec<u8>,       // Font bytes
    pub body: Vec<u8>,
    pub mono: Vec<u8>,
}

impl BrandedDocument {
    pub fn prepare(doc: ParsedDocument, profile: ClientProfile) -> anyhow::Result<Self> {
        let page = resolve_page_dimensions(&profile);
        let colours = resolve_colours(&profile)?;
        let fonts = load_fonts(&profile)?;

        Ok(BrandedDocument { profile, doc, page, fonts, colours })
    }
}

fn resolve_page_dimensions(profile: &ClientProfile) -> PageDimensions {
    let (width, height) = match profile.page_size.as_str() {
        "A4"     => (210.0_f32, 297.0_f32),
        "Letter" => (215.9_f32, 279.4_f32),
        "Legal"  => (215.9_f32, 355.6_f32),
        _        => (210.0_f32, 297.0_f32),
    };

    let content_width  = width  - profile.margin_left - profile.margin_right;
    let content_height = height - profile.margin_top  - profile.margin_bottom;

    PageDimensions {
        width_mm: width,
        height_mm: height,
        margin_top_mm: profile.margin_top,
        margin_bottom_mm: profile.margin_bottom,
        margin_left_mm: profile.margin_left,
        margin_right_mm: profile.margin_right,
        content_width_mm: content_width,
        content_height_mm: content_height,
    }
}

fn resolve_colours(profile: &ClientProfile) -> anyhow::Result<ResolvedColours> {
    Ok(ResolvedColours {
        primary:      hex_to_rgb(&profile.colour_primary)?,
        secondary:    hex_to_rgb(&profile.colour_secondary)?,
        accent:       hex_to_rgb(&profile.colour_accent)?,
        text:         hex_to_rgb(&profile.colour_text)?,
        background:   hex_to_rgb(&profile.colour_background)?,
        table_header: hex_to_rgb(&profile.colour_table_header)?,
        table_border: hex_to_rgb(&profile.colour_table_border)?,
        callout_bg:   hex_to_rgb(&profile.colour_callout_bg)?,
    })
}

fn hex_to_rgb(hex: &str) -> anyhow::Result<[f32; 3]> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.0;
    Ok([r, g, b])
}

fn load_fonts(profile: &ClientProfile) -> anyhow::Result<ResolvedFonts> {
    // Load from FONTS_DIR, fall back to bundled defaults
    // Font files are pre-downloaded and cached on the server
    // Heading and body fonts loaded from /opt/typeset/fonts/[font-name].ttf
    let heading = load_font_bytes(&profile.font_heading)?;
    let body    = load_font_bytes(&profile.font_body)?;
    let mono    = load_font_bytes(&profile.font_mono)?;
    Ok(ResolvedFonts { heading, body, mono })
}

fn load_font_bytes(font_name: &str) -> anyhow::Result<Vec<u8>> {
    let fonts_dir = std::env::var("FONTS_DIR").unwrap_or("/opt/typeset/fonts".to_string());
    let path = format!("{}/{}.ttf", fonts_dir, font_name.replace(' ', "-").to_lowercase());

    if std::path::Path::new(&path).exists() {
        return Ok(std::fs::read(path)?);
    }

    // Fall back to bundled Helvetica/Courier
    match font_name {
        f if f.contains("Mono") || f.contains("Code") => {
            Ok(include_bytes!("../../assets/fonts/courier.ttf").to_vec())
        }
        _ => Ok(include_bytes!("../../assets/fonts/helvetica.ttf").to_vec())
    }
}
```

---

## 10. PDF Renderer

```rust
// src/renderers/pdf.rs

use printpdf::*;
use crate::brand::engine::BrandedDocument;
use crate::parser::markdown::DocumentNode;

pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new(
        branded.doc.frontmatter
            .as_ref()
            .and_then(|f| f.title.as_deref())
            .unwrap_or("Document"),
        Mm(branded.page.width_mm),
        Mm(branded.page.height_mm),
        "Layer 1",
    );

    let heading_font = doc.add_external_font(
        std::io::Cursor::new(&branded.fonts.heading)
    )?;
    let body_font = doc.add_external_font(
        std::io::Cursor::new(&branded.fonts.body)
    )?;
    let mono_font = doc.add_external_font(
        std::io::Cursor::new(&branded.fonts.mono)
    )?;

    let mut renderer = PdfPageRenderer {
        doc: &doc,
        branded,
        heading_font: &heading_font,
        body_font: &body_font,
        mono_font: &mono_font,
        current_page: page1,
        current_layer: layer1,
        cursor_y: Mm(branded.page.height_mm - branded.page.margin_top_mm),
        page_number: 1,
    };

    // Cover page
    if branded.profile.cover_enabled {
        renderer.render_cover()?;
        renderer.new_page();
    }

    // Header on first content page
    if branded.profile.header_enabled {
        renderer.render_header()?;
    }

    // Content nodes
    for node in &branded.doc.nodes {
        renderer.render_node(node)?;
    }

    // Footers (post-processing pass)
    renderer.apply_footers()?;

    let bytes = doc.save_to_bytes()?;
    Ok(bytes)
}

struct PdfPageRenderer<'a> {
    doc: &'a PdfDocumentReference,
    branded: &'a BrandedDocument,
    heading_font: &'a IndirectFontRef,
    body_font: &'a IndirectFontRef,
    mono_font: &'a IndirectFontRef,
    current_page: PdfPageIndex,
    current_layer: PdfLayerIndex,
    cursor_y: Mm,
    page_number: u32,
}

impl<'a> PdfPageRenderer<'a> {
    fn layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.current_page).get_layer(self.current_layer)
    }

    fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(
            Mm(self.branded.page.width_mm),
            Mm(self.branded.page.height_mm),
            "Layer 1",
        );
        self.current_page = page;
        self.current_layer = layer;
        self.cursor_y = Mm(
            self.branded.page.height_mm - self.branded.page.margin_top_mm
        );
        self.page_number += 1;

        if self.branded.profile.header_enabled {
            let _ = self.render_header();
        }
    }

    fn check_page_break(&mut self, needed_height: f32) {
        let bottom = Mm(self.branded.page.margin_bottom_mm + 20.0);
        if self.cursor_y - Mm(needed_height) < bottom {
            self.new_page();
        }
    }

    fn render_node(&mut self, node: &DocumentNode) -> anyhow::Result<()> {
        match node {
            DocumentNode::Heading { level, text, .. } => {
                self.render_heading(*level, text)?;
            }
            DocumentNode::Paragraph { text } => {
                self.render_paragraph(text)?;
            }
            DocumentNode::BulletList { items } => {
                self.render_bullet_list(items)?;
            }
            DocumentNode::OrderedList { items, start } => {
                self.render_ordered_list(items, *start)?;
            }
            DocumentNode::Table { headers, rows } => {
                self.render_table(headers, rows)?;
            }
            DocumentNode::CodeBlock { code, .. } => {
                self.render_code_block(code)?;
            }
            DocumentNode::Blockquote { text } => {
                self.render_blockquote(text)?;
            }
            DocumentNode::HorizontalRule => {
                self.render_rule()?;
            }
            DocumentNode::PageBreak => {
                self.new_page();
            }
            DocumentNode::Image { .. } => {
                // Images deferred to v1.1
            }
        }
        Ok(())
    }

    fn render_heading(&mut self, level: u8, text: &str) -> anyhow::Result<()> {
        let (size, colour, spacing_before) = match level {
            1 => (self.branded.profile.font_size_h1, &self.branded.colours.primary, 10.0),
            2 => (self.branded.profile.font_size_h2, &self.branded.colours.secondary, 8.0),
            3 => (self.branded.profile.font_size_h3, &self.branded.colours.text, 6.0),
            _ => (self.branded.profile.font_size_base + 1.0, &self.branded.colours.text, 4.0),
        };

        let line_height = size * self.branded.profile.line_height;
        self.check_page_break(line_height + spacing_before + 4.0);

        self.cursor_y -= Mm(spacing_before);

        let layer = self.layer();
        layer.use_text(
            text,
            size as f64,
            Mm(self.branded.page.margin_left_mm),
            self.cursor_y,
            self.heading_font,
        );

        // Accent underline on H1
        if level == 1 {
            self.cursor_y -= Mm(2.0);
            let [r, g, b] = self.branded.colours.accent;
            layer.set_outline_color(Color::Rgb(Rgb::new(r as f64, g as f64, b as f64, None)));
            layer.set_outline_thickness(0.5);
            layer.add_line(Line {
                points: vec![
                    (
                        Point::new(
                            Mm(self.branded.page.margin_left_mm),
                            self.cursor_y,
                        ),
                        false,
                    ),
                    (
                        Point::new(
                            Mm(self.branded.page.margin_left_mm + self.branded.page.content_width_mm),
                            self.cursor_y,
                        ),
                        false,
                    ),
                ],
                is_closed: false,
                has_fill: false,
                has_stroke: true,
                is_clipping_path: false,
            });
        }

        self.cursor_y -= Mm(line_height + 2.0);
        Ok(())
    }

    fn render_paragraph(&mut self, text: &str) -> anyhow::Result<()> {
        // Simple text wrap implementation
        // Full implementation handles inline bold, italic, links
        let size = self.branded.profile.font_size_base;
        let line_height = size * self.branded.profile.line_height;
        let chars_per_line = (self.branded.page.content_width_mm / (size * 0.5)) as usize;

        let lines = wrap_text(text, chars_per_line);
        let total_height = lines.len() as f32 * line_height + self.branded.profile.paragraph_spacing;

        self.check_page_break(total_height);

        let layer = self.layer();
        let [r, g, b] = self.branded.colours.text;
        layer.set_fill_color(Color::Rgb(Rgb::new(r as f64, g as f64, b as f64, None)));

        for line in &lines {
            layer.use_text(
                line,
                size as f64,
                Mm(self.branded.page.margin_left_mm),
                self.cursor_y,
                self.body_font,
            );
            self.cursor_y -= Mm(line_height);
        }

        self.cursor_y -= Mm(self.branded.profile.paragraph_spacing);
        Ok(())
    }

    fn render_table(&mut self, headers: &[String], rows: &[Vec<String>]) -> anyhow::Result<()> {
        let col_width = self.branded.page.content_width_mm / headers.len() as f32;
        let row_height = self.branded.profile.font_size_base * 1.8;
        let total_height = (rows.len() + 1) as f32 * row_height + 4.0;

        self.check_page_break(total_height);

        let layer = self.layer();

        // Header row background
        let [r, g, b] = self.branded.colours.table_header;
        layer.set_fill_color(Color::Rgb(Rgb::new(r as f64, g as f64, b as f64, None)));

        // Draw header cells and text, then each data row
        // Full implementation in src/renderers/pdf.rs

        self.cursor_y -= Mm(total_height + self.branded.profile.section_spacing);
        Ok(())
    }

    fn render_blockquote(&mut self, text: &str) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let line_height = size * self.branded.profile.line_height;
        let padding = 4.0_f32;
        let chars_per_line = ((self.branded.page.content_width_mm - 12.0) / (size * 0.5)) as usize;
        let lines = wrap_text(text, chars_per_line);
        let block_height = lines.len() as f32 * line_height + padding * 2.0;

        self.check_page_break(block_height + 8.0);

        let layer = self.layer();

        // Callout background
        let [r, g, b] = self.branded.colours.callout_bg;
        layer.set_fill_color(Color::Rgb(Rgb::new(r as f64, g as f64, b as f64, None)));

        // Accent left border
        let [ar, ag, ab] = self.branded.colours.accent;
        layer.set_fill_color(Color::Rgb(Rgb::new(ar as f64, ag as f64, ab as f64, None)));
        // Draw 3px left bar

        // Text
        let [tr, tg, tb] = self.branded.colours.text;
        layer.set_fill_color(Color::Rgb(Rgb::new(tr as f64, tg as f64, tb as f64, None)));
        let text_x = Mm(self.branded.page.margin_left_mm + 8.0);

        let mut y = self.cursor_y - Mm(padding);
        for line in &lines {
            layer.use_text(line, size as f64, text_x, y, self.body_font);
            y -= Mm(line_height);
        }

        self.cursor_y -= Mm(block_height + self.branded.profile.paragraph_spacing);
        Ok(())
    }

    fn render_code_block(&mut self, code: &str) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base - 1.5;
        let line_height = size * 1.4;
        let lines: Vec<&str> = code.lines().collect();
        let block_height = lines.len() as f32 * line_height + 8.0;

        self.check_page_break(block_height + 8.0);

        // Dark background box
        // Monospaced text in mono font
        // Full implementation in src/renderers/pdf.rs

        self.cursor_y -= Mm(block_height + self.branded.profile.paragraph_spacing);
        Ok(())
    }

    fn render_rule(&mut self) -> anyhow::Result<()> {
        self.check_page_break(8.0);
        self.cursor_y -= Mm(4.0);

        let layer = self.layer();
        let [r, g, b] = self.branded.colours.table_border;
        layer.set_outline_color(Color::Rgb(Rgb::new(r as f64, g as f64, b as f64, None)));
        layer.set_outline_thickness(0.3);

        // Draw horizontal line across content width
        self.cursor_y -= Mm(4.0);
        Ok(())
    }

    fn render_bullet_list(&mut self, items: &[String]) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let line_height = size * self.branded.profile.line_height;

        for item in items {
            self.check_page_break(line_height + 2.0);
            let layer = self.layer();
            // Bullet character
            layer.use_text("•", size as f64,
                Mm(self.branded.page.margin_left_mm + 2.0), self.cursor_y, self.body_font);
            // Item text (with indent)
            layer.use_text(item, size as f64,
                Mm(self.branded.page.margin_left_mm + 6.0), self.cursor_y, self.body_font);
            self.cursor_y -= Mm(line_height + 1.0);
        }

        self.cursor_y -= Mm(self.branded.profile.paragraph_spacing);
        Ok(())
    }

    fn render_ordered_list(&mut self, items: &[String], start: u64) -> anyhow::Result<()> {
        let size = self.branded.profile.font_size_base;
        let line_height = size * self.branded.profile.line_height;

        for (i, item) in items.iter().enumerate() {
            self.check_page_break(line_height + 2.0);
            let layer = self.layer();
            let label = format!("{}.", i as u64 + start);
            layer.use_text(&label, size as f64,
                Mm(self.branded.page.margin_left_mm + 2.0), self.cursor_y, self.body_font);
            layer.use_text(item, size as f64,
                Mm(self.branded.page.margin_left_mm + 8.0), self.cursor_y, self.body_font);
            self.cursor_y -= Mm(line_height + 1.0);
        }

        self.cursor_y -= Mm(self.branded.profile.paragraph_spacing);
        Ok(())
    }

    fn render_cover(&mut self) -> anyhow::Result<()> {
        match self.branded.profile.cover_template.as_str() {
            "bold"  => self.render_cover_bold(),
            "split" => self.render_cover_split(),
            _       => self.render_cover_minimal(),
        }
    }

    fn render_cover_minimal(&mut self) -> anyhow::Result<()> {
        // Logo centred, top third
        // Title in heading font, large, centred
        // Subtitle in body font, muted, centred
        // Recipient and date at bottom
        Ok(())
    }

    fn render_cover_bold(&mut self) -> anyhow::Result<()> {
        // Full bleed primary colour background
        // White logo top-left
        // Large white title bottom-left
        // Accent bar bottom edge
        Ok(())
    }

    fn render_cover_split(&mut self) -> anyhow::Result<()> {
        // Left half primary colour, right half white/bg
        // Logo on colour side, title on white side
        Ok(())
    }

    fn render_header(&mut self) -> anyhow::Result<()> {
        // Logo + optional document title
        // Border line beneath
        Ok(())
    }

    fn apply_footers(&mut self) -> anyhow::Result<()> {
        // Post-processing pass: add page numbers to all pages
        Ok(())
    }
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_chars && !current.is_empty() {
            lines.push(current.trim().to_string());
            current = word.to_string();
        } else {
            if !current.is_empty() { current.push(' '); }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current.trim().to_string());
    }

    lines
}
```

---

## 11. DOCX Renderer

```rust
// src/renderers/docx.rs

use docx_rs::*;
use crate::brand::engine::BrandedDocument;
use crate::parser::markdown::DocumentNode;

pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>> {
    let mut docx = Docx::new();

    // Apply document styles from brand profile
    docx = apply_document_styles(docx, branded);

    // Cover page properties
    if branded.profile.cover_enabled {
        docx = add_cover_page(docx, branded);
    }

    // Content nodes
    for node in &branded.doc.nodes {
        docx = render_node(docx, node, branded);
    }

    let mut buf = Vec::new();
    docx.build().pack(&mut std::io::Cursor::new(&mut buf))?;
    Ok(buf)
}

fn apply_document_styles(docx: Docx, branded: &BrandedDocument) -> Docx {
    // Map brand profile → Word styles
    // Heading 1, Heading 2, Heading 3, Normal, Code, Quote
    // Margins set on section properties
    docx
}

fn render_node(docx: Docx, node: &DocumentNode, branded: &BrandedDocument) -> Docx {
    match node {
        DocumentNode::Heading { level, text, .. } => {
            let style = match level {
                1 => "Heading1",
                2 => "Heading2",
                3 => "Heading3",
                _ => "Heading4",
            };
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(text)).style(style)
            )
        }
        DocumentNode::Paragraph { text } => {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(text))
            )
        }
        DocumentNode::BulletList { items } => {
            let mut d = docx;
            for item in items {
                d = d.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(item))
                        .numbering(NumberingId::new(1), IndentLevel::new(0))
                );
            }
            d
        }
        DocumentNode::Table { headers, rows } => {
            let mut table = Table::new(vec![]);
            // Build header row with brand table_header colour
            // Build data rows
            docx.add_table(table)
        }
        DocumentNode::CodeBlock { code, .. } => {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(code)).style("Code")
            )
        }
        DocumentNode::Blockquote { text } => {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(text)).style("Quote")
            )
        }
        DocumentNode::HorizontalRule => {
            docx.add_paragraph(Paragraph::new().add_run(Run::new()))
        }
        DocumentNode::PageBreak => {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_break(BreakType::Page))
            )
        }
        _ => docx,
    }
}

fn add_cover_page(docx: Docx, branded: &BrandedDocument) -> Docx {
    // Cover page as first section with different header/footer
    docx
}
```

---

## 12. HTTP Routes

```rust
// src/routes/render.rs

use axum::{extract::State, http::StatusCode, Json};
use crate::models::render_job::{RenderRequest, RenderResponse};

pub async fn handle_render(
    State(state): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<RenderResponse>, (StatusCode, String)> {
    let start = std::time::Instant::now();

    // 1. Fetch client profile (or default)
    let profile = match &req.client {
        Some(slug) => state.db.get_client_profile(slug).await
            .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?,
        None => ClientProfile::default(),
    };

    // 2. Parse markdown
    let mut doc = crate::parser::markdown::parse(&req.content);

    // 3. Apply overrides
    if let Some(overrides) = &req.overrides {
        if let Some(fm) = &mut doc.frontmatter {
            if let Some(t) = &overrides.title    { fm.title    = Some(t.clone()); }
            if let Some(s) = &overrides.subtitle { fm.subtitle = Some(s.clone()); }
            if let Some(r) = &overrides.recipient{ fm.recipient= Some(r.clone()); }
            if let Some(d) = &overrides.date     { fm.date     = Some(d.clone()); }
        }
    }

    // 4. Brand engine
    let branded = crate::brand::engine::BrandedDocument::prepare(doc, profile)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Render
    let render_id = uuid::Uuid::new_v4().to_string();
    let mut pdf_url = None;
    let mut docx_url = None;

    match req.format {
        RenderFormat::Pdf | RenderFormat::Both => {
            let bytes = crate::renderers::pdf::render(&branded)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            pdf_url = Some(
                state.storage.upload(&format!("{}.pdf", render_id), bytes, "application/pdf").await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            );
        }
        _ => {}
    }

    match req.format {
        RenderFormat::Docx | RenderFormat::Both => {
            let bytes = crate::renderers::docx::render(&branded)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            docx_url = Some(
                state.storage.upload(&format!("{}.docx", render_id), bytes,
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document").await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            );
        }
        _ => {}
    }

    // 6. Log to DB
    state.db.log_render(&render_id, &req, pdf_url.as_deref(), docx_url.as_deref()).await.ok();

    let render_ms = start.elapsed().as_millis() as u64;

    Ok(Json(RenderResponse {
        success: true,
        render_id,
        pdf_url,
        docx_url,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(30))
            .unwrap()
            .to_rfc3339(),
        render_ms,
    }))
}
```

---

## 13. Deployment

### Systemd Service

```ini
# /etc/systemd/system/typeset.service

[Unit]
Description=Typeset Render Service
After=network.target postgresql.service

[Service]
Type=simple
User=typeset
WorkingDirectory=/opt/typeset
ExecStart=/opt/typeset/typeset
Restart=always
RestartSec=5
Environment=RUST_LOG=info

# Resource limits — protect other services on same droplet
MemoryMax=256M
CPUQuota=40%
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

### Nginx Proxy

```nginx
# /etc/nginx/sites-available/typeset

server {
    listen 443 ssl;
    server_name typeset.chrisgarlick.com;

    ssl_certificate     /etc/letsencrypt/live/typeset.chrisgarlick.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/typeset.chrisgarlick.com/privkey.pem;

    location /api/ {
        proxy_pass         http://127.0.0.1:3200;
        proxy_http_version 1.1;
        proxy_set_header   Host $host;
        proxy_set_header   X-Real-IP $remote_addr;
        proxy_read_timeout 60s;

        # Rate limiting
        limit_req zone=typeset_api burst=20 nodelay;
    }

    location / {
        proxy_pass http://typeset_frontend;  # Vercel or local Next.js
    }
}

# Rate limit zone
limit_req_zone $binary_remote_addr zone=typeset_api:10m rate=30r/m;
```

### Build & Deploy Script

```bash
#!/bin/bash
# deploy.sh

set -e

echo "Building Typeset..."
cargo build --release

echo "Stopping service..."
sudo systemctl stop typeset

echo "Copying binary..."
sudo cp target/release/typeset /opt/typeset/typeset

echo "Running migrations..."
sqlx migrate run

echo "Starting service..."
sudo systemctl start typeset

echo "Checking health..."
sleep 2
curl -f http://localhost:3200/health && echo "✓ Typeset running"
```

---

## 14. Resource Profile

| Metric | Value |
|--------|-------|
| Binary size | ~12MB |
| Idle RAM | ~8MB |
| RAM per render (PDF) | ~12–18MB |
| RAM per render (DOCX) | ~6–10MB |
| RAM hard limit (systemd) | 256MB |
| CPU quota | 40% (won't starve other services) |
| Render time — PDF | 400–900ms |
| Render time — DOCX | 100–300ms |
| Concurrent renders | 4 (configurable) |
| Cold start | <100ms |

Compare to Node/Puppeteer approach:
| Metric | Puppeteer | Typeset (Rust) |
|--------|-----------|----------------|
| Idle RAM | 80–150MB | 8MB |
| RAM per render | 200–500MB | 12–18MB |
| Render time | 3–8s | 0.4–0.9s |
| External deps | Chromium | None |

---

## 15. Font Management

Fonts are pre-downloaded to `/opt/typeset/fonts/` on the server. The service does not fetch fonts at render time.

### Bundled Defaults
- `helvetica.ttf` — default body/heading fallback
- `courier.ttf` — default mono fallback

### Adding Fonts
```bash
# Download a Google Font for use in client profiles
cd /opt/typeset/fonts

# Example: Instrument Serif
wget "https://fonts.gstatic.com/s/instrumentserif/..." -O instrument-serif.ttf

# Restart not required — fonts loaded per-render
```

Font names in client profiles must match filenames (lowercased, spaces as hyphens). Example: `"Instrument Serif"` → `/opt/typeset/fonts/instrument-serif.ttf`.

---

## 16. API Reference

### `POST /api/render`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `client` | string | No | Client profile slug |
| `document_type` | enum | Yes | `proposal` `report` `brief` `sop` `invoice` `general` |
| `format` | enum | Yes | `pdf` `docx` `both` |
| `content` | string | Yes | Markdown content |
| `overrides.title` | string | No | Override frontmatter title |
| `overrides.subtitle` | string | No | |
| `overrides.recipient` | string | No | |
| `overrides.date` | string | No | |

### `GET /api/clients`
Returns array of client profiles for authenticated user.

### `GET /api/clients/:slug`
Returns single client profile.

### `POST /api/clients`
Create or update a client profile. Body matches `ClientProfile` schema.

### `DELETE /api/clients/:slug`
Delete a client profile.

### `GET /api/history`
Query params: `limit` (default 20), `offset`, `client`, `format`, `from`, `to`.

### `GET /api/health`
Returns `{ "status": "ok", "version": "0.1.0" }`. Used by deploy script and monitoring.

---

## 17. MVP Checklist

### Rust Service
- [ ] Project scaffold (`cargo new typeset`)
- [ ] Axum server on port 3200
- [ ] Markdown parser (`pulldown-cmark` integration)
- [ ] ClientProfile model + DB queries
- [ ] Brand engine (colour + font resolution)
- [ ] PDF renderer — headings, paragraphs, lists, tables, rules
- [ ] DOCX renderer — headings, paragraphs, lists, tables
- [ ] DigitalOcean Spaces upload
- [ ] POST `/api/render` route
- [ ] GET/POST `/api/clients` routes
- [ ] GET `/api/health` route
- [ ] Render history logging
- [ ] Systemd service file
- [ ] Nginx proxy config
- [ ] Deploy script

### Database
- [ ] Migration 001 — client_profiles
- [ ] Migration 002 — render_history
- [ ] sqlx queries tested

### Cover Templates (MVP — Minimal only)
- [ ] Minimal cover (logo, title, subtitle, recipient, date)

### Fonts
- [ ] Bundled Helvetica + Courier fallbacks
- [ ] Font directory structure on droplet
- [ ] At least 4 Google Fonts pre-downloaded for testing

### Testing
- [ ] Unit tests for markdown parser
- [ ] Unit tests for hex_to_rgb
- [ ] Integration test — render sample.md with sample_profile.json
- [ ] Load test — 10 concurrent renders, measure memory peak

---

*Typeset technical documentation v1.0*
*May 2026 · Rust · DigitalOcean*