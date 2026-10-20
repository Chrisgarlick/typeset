# Typeset

Markdown + YAML frontmatter → branded PDF (or DOCX), via a stateless HTTP API.

- **Live API:** `https://typeset.chrisgarlick.com`
- **API reference:** [`API.md`](./API.md)
- **HTML output proposal (future):** [`HTML.md`](./HTML.md)

---

# Authoring rules

> **This is the contract.** If you're writing code or an LLM prompt that generates markdown for Typeset, these rules describe exactly what the rendered PDF will look like. They are enforced by the renderer, not aspirational — page breaks, orphan protection, and code-block behaviour all follow from these rules deterministically.

## The five rules

| # | Rule | What it does | How to invoke it |
|---|------|--------------|------------------|
| 1 | **Document title → cover page** | First page of the PDF is a full branded cover, header/footer suppressed | Set `title:` in YAML frontmatter, **or** open the document with `# Title` |
| 2 | **The top heading level opens a numbered section** | Each section opens below a heavy rule with its number (01, 02…) in the margin column. It starts a new page only when less than about a third of the current page is left, so short sections don't leave half-empty pages | Use `## Section Name` (or `#` if the body uses H1s) |
| 3 | **Headings are sticky** | Every heading stays with the content that follows it — never stranded at the bottom of a page | Use `### Subsection` inside a section |
| 4 | **Code blocks, cards and callouts stay whole** | They move to the next page rather than split | Standard fenced code: ` ``` ` |
| 5 | **Tables size their columns to the content** | Short columns fit their content; long ones share the rest. Tables with long text span the full width, including the margin column. Long tables split between rows, repeating the header | Standard GFM table syntax |

## Cover page precedence

Deterministic — no ambiguity:

```
if frontmatter.title is set:
    cover.title = frontmatter.title
    body H1s (if any) are the sections
elif body contains an H1:
    cover.title = first H1 in the body (consumed — not rendered again)
else:
    no cover; document flows from page 1
```

The cover always uses these frontmatter fields when present:

| Field | Used as |
|-------|---------|
| `title` | Cover title (large, primary colour) |
| `subtitle` | Below title |
| `recipient` | "PREPARED FOR" line in the metadata pillar |
| `author` | "BY" line |
| `date` | "DATE" line |

Missing fields are silently skipped. The running head on body pages shows the document title on the left and the current section on the right. The footer shows "Prepared for <recipient>" (or the title) and "Page n of N". Sections become PDF bookmarks; title, author and subtitle become the PDF's metadata.

## Heading semantics

For SEO and HTML re-use, treat the document as having **one logical `<h1>`** — the cover title.

| Markdown | Page behaviour | Visual weight |
|----------|---------------|----------------|
Sizes are relative to the section level (the shallowest heading level in the body, after the cover takes its title), so a document whose sections are `##` looks the same as one whose sections are `#`.

| Level | Page behaviour | Visual weight |
|----------|---------------|----------------|
| Section (usually `##`) | New page only if the current one is mostly used | Largest, heavy rule above, number in the margin |
| One below (`###`) | Inline, sticky to next block | Medium |
| Two below (`####`) | Inline, sticky to next block | Small heading |
| Deeper | Inline, sticky to next block | Body size, semibold |

Headings inside a styled section or column (cards, callouts) are card titles: small, in the box's text colour, and left out of the bookmarks.

**Author mental model:** "`##` opens a section, `###` stays inside it. Never think about page breaks — the headings declare structure, and the renderer decides where pages end."

## Code blocks

````markdown
```
plain code
```

```python
syntax highlighted
```

```fletcher
diagram code (see below)
```
````

- Fenced code blocks use a dark background and monospace font.
- Blocks stay whole and move to the next page rather than split. Keep them under a page long.
- Language tags follow standard Markdown (`python`, `yaml`, `json`, `rust`, etc.).
- The special language `fletcher` activates diagram rendering — see below.

## Tables

Standard GitHub-flavoured Markdown pipe tables:

```markdown
| Column A | Column B |
|----------|----------|
| Value 1  | Value 2  |
```

Tables don't split mid-row. Large tables that exceed one page will split, with the header row repeating on each continuation page.

## Lists

```markdown
- Bullet item
- Bullet item

1. Numbered item
2. Numbered item
```

Bullets and numbers are rendered in the accent colour.

## Diagrams (Fletcher)

For flowcharts, sequence diagrams, and node/edge graphs, use a `fletcher` code block. The contents are passed to the [Typst `fletcher` package](https://typst.app/universe/package/fletcher/) and rendered inline.

````markdown
## Architecture

```fletcher
node((0, 0), [Client]),
node((1, 0), [API]),
node((2, 0), [Database]),
edge((0, 0), (1, 0), "->", [HTTP]),
edge((1, 0), (2, 0), "->", [SQL]),
```
````

Each line inside the fenced block is a typst function call (`node`, `edge`, etc.) — note the trailing commas. The renderer wraps them in `#diagram(...)`. See the [fletcher docs](https://typst.app/universe/package/fletcher/) for the full DSL.

## Frontmatter

Single-line YAML at the very top of the markdown, fenced by `---`:

```markdown
---
title: Engagement Proposal
subtitle: Q3 2026 Discovery
recipient: Acme Corp
date: 2026-05-12
author: Chris Garlick
document_type: proposal
client: acme
---

# Body starts here…
```

**Supported keys (everything else is ignored):**

| Key | Type | Purpose |
|-----|------|---------|
| `title` | string | Cover title |
| `subtitle` | string | Cover subtitle |
| `recipient` | string | "PREPARED FOR" |
| `author` | string | "BY" |
| `date` | string | "DATE" (free-form) |
| `document_type` | string | Categorisation only (used for logging) |
| `client` | string | Client profile slug for branding |

Restrictions: single-line `key: value` only. No nested objects, no arrays, no multi-line strings. Surrounding quotes (`"` or `'`) are tolerated.

## Other markdown elements

| Element | Markdown | Behaviour |
|---------|----------|-----------|
| Bold | `**text**` | Bold inline |
| Italic | `*text*` | Italic inline |
| Inline code | `` `text` `` | Monospace with subtle background |
| Strikethrough | `~~text~~` | Struck through |
| Links | `[label](url)` | Accent colour |
| Blockquote | `> text` | Indented quote block |
| Horizontal rule | `---` (outside frontmatter) | Thin divider, **not** a page break |
| Image | `![alt](url)` | Renders if accessible to the typesetter (local files only for now) |

To force a page break, put `{{pagebreak}}` alone on a line. Use it rarely: the renderer already starts a section on a new page when the current one is mostly used. In JSON documents, each entry in `pages` is a section boundary with the same rule, and a `page_break` block forces one.

---

# How to generate Typeset-ready markdown

If you're a downstream system generating markdown to feed into Typeset, follow this skeleton:

```markdown
---
title: <document title>
subtitle: <optional one-liner>
recipient: <who it's for>
author: <who wrote it>
date: <YYYY-MM-DD or free-form>
---

## <First section heading>

<Body paragraphs, lists, tables, code blocks…>

### <Inline subsection if needed>

<more content>

## <Next section>

<…>
```

**Key generation guidance for LLMs / templating engines:**

1. **Set `title` in frontmatter.** Don't open with `# Title` unless you specifically want the title rendered both as cover and as the first H1 in the body.
2. **Use `##` for sections.** Each one is numbered and bookmarked. Don't use `#` in the body — that's reserved for the title.
3. **Use `###` for subsections.** Multiple `###`s can live under one `##`.
4. **Don't insert manual page-break markers.** The renderer places page breaks.
5. **Keep code blocks under a page.** They don't split.
6. **For diagrams, use ` ```fletcher ` blocks.** Don't try to render ASCII art.
7. **Frontmatter is single-line YAML only.** Strip multi-line strings or arrays before generating.

---

# API in one paragraph

`POST /api/render` with `Authorization: Bearer $TYPESET_TOKEN` (single shared secret, set via env var on the server) and a JSON body of `{ "content": "<markdown>", "document_type": "proposal|report|brief|sop|invoice|general", "format": "pdf|docx", "client": "<optional-slug>" }`. Response is the raw PDF or DOCX bytes. Render endpoints are rate-limited to **10 req/min per IP** (configurable via `TYPESET_RATE_LIMIT_PER_MIN`). See [`API.md`](./API.md) for the full surface.

---

# Local development

```bash
# Prerequisites: Rust 1.95+, Postgres, Typst CLI (`brew install typst`)
cargo run                       # starts API on :3200
cargo test --release            # unit + integration tests
cargo test --release smoke_     # render-to-PDF smoke tests (need typst on PATH)

# Render a local file for design review (writes the PDF and its .typ source)
TYPESET_PREVIEW_IN=examples/ai-strategy.json TYPESET_PREVIEW_OUT=/tmp/out.pdf \
  TYPESET_PREVIEW_PROFILE=profile-overrides.json cargo test preview -- --ignored
```

The page design lives in `src/renderers/theme.typ`; the renderer emits the design tokens (`t`) and document facts (`meta`) above it.

# Deployment

GitHub Actions builds and pushes a Docker image to `ghcr.io/chrisgarlick/typeset/api:latest` on every push to `main`. Run `./deploy.sh` to pull and restart on the production host.

The runtime image bundles:
- The Rust binary
- Typst CLI (pinned version, see `Dockerfile`)
- Bundled fonts from `assets/fonts` (Inter, Inter Tight, JetBrains Mono); Typst uses only these
- A pre-cached copy of the Fletcher diagram package (so first render after deploy doesn't pay a network round-trip)
