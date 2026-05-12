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
| 2 | **`##` H2 starts a new page** | Each H2 = a new "section" on its own page | Use `## Section Name` |
| 3 | **`###` and deeper headings are sticky** | H3–H6 stay glued to the content that follows them — never orphaned at page bottom | Use `### Subsection` inside a page |
| 4 | **Code blocks split cleanly across pages** | Block stays whole if it fits; otherwise splits with the dark background repeating on each page | Standard fenced code: ` ``` ` |
| 5 | **Tables and lists stay with their context** | Tables don't split unless they have to. Lists stick to the paragraph that introduces them. | Standard GFM table / list syntax |

## Cover page precedence

Deterministic — no ambiguity:

```
if frontmatter.title is set:
    cover.title = frontmatter.title
    body H1 (if any) renders inline as the first section (also starts a new page)
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

Missing fields are silently skipped. Header text on body pages is always the document title.

## Heading semantics

For SEO and HTML re-use, treat the document as having **one logical `<h1>`** — the cover title.

| Markdown | Page behaviour | Visual weight |
|----------|---------------|----------------|
| `# Title` | Cover OR new page (see precedence above) | Largest, primary colour, accent underline |
| `## Section` | **Always starts a new page** | Large, secondary colour |
| `### Subsection` | Inline, sticky to next block | Medium, body colour |
| `#### …` | Inline, sticky to next block | Slightly larger than body |
| `##### / ######` | Inline, sticky to next block | Body weight |

**Author mental model:** "Promote to `##` to break the page. Demote to `###` to stay inline. Never think about explicit page breaks — the headings declare structure."

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
- Short blocks stay whole. Long blocks split across pages — the dark background and text colour repeat on each page automatically.
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

Bullets are rendered in the accent colour. Lists are sticky — the first item won't appear orphaned on a new page if the introducing paragraph was on the previous page.

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

To force an explicit page break that isn't tied to a heading, insert a single `---` line **only if you really mean it** — this currently renders as a horizontal rule, not a page break. Page breaks are deliberately tied to `##` headings so document structure stays visible in the source.

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

## <Next section — automatically new page>

<…>
```

**Key generation guidance for LLMs / templating engines:**

1. **Set `title` in frontmatter.** Don't open with `# Title` unless you specifically want the title rendered both as cover and as the first H1 in the body.
2. **Use `##` to chunk pages.** Want six pages? Write six `##` sections. Don't use `#` to break pages — that's reserved for the title.
3. **Use `###` for subsections within a page.** Multiple `###`s can live under one `##`.
4. **Don't insert manual page-break markers.** Promote a heading instead.
5. **Long code in a `##` section is fine** — it'll split with the background intact. No need to chunk it artificially.
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
```

# Deployment

GitHub Actions builds and pushes a Docker image to `ghcr.io/chrisgarlick/typeset/api:latest` on every push to `main`. Run `./deploy.sh` to pull and restart on the production host.

The runtime image bundles:
- The Rust binary
- Typst CLI (pinned version, see `Dockerfile`)
- DejaVu + Inter fonts
- A pre-cached copy of the Fletcher diagram package (so first render after deploy doesn't pay a network round-trip)
