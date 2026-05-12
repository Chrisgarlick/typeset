# Typeset HTML Rendering — Proposal

> **Status: not implemented.** This document describes a *proposed* future feature for rendering markdown into interactive HTML alongside the existing PDF/DOCX output. Nothing in here is live yet — the live API surface is documented in `API.md`.

The goal: same input (markdown + YAML frontmatter + optional client profile), but with `format: "html"` returning a self-contained, branded, interactive HTML document instead of a binary file.

---

## Why add HTML?

PDF and DOCX are great for "final deliverable" outputs, but they're frozen. Adding HTML opens up:

- **Live preview** in an iframe without needing a PDF viewer plugin
- **Interactive elements** the print formats can't express (collapsible sections, tabs, copy-to-clipboard, anchored ToC, dark mode toggle)
- **Easy embedding** in dashboards, client portals, or email summaries
- **Cheap rendering** — HTML generation skips the heavy `printpdf` layout pass, so it's faster and uses less memory than PDF for the same content

The frontmatter and client-profile model already in place maps cleanly onto HTML — colours become CSS variables, fonts become `font-family` declarations, margins become page-level padding.

---

## Proposed API surface

### Option A — extend the existing `format` enum (preferred)

Add `"html"` to `RenderFormat` in `src/models/render_job.rs`. Existing callers are unaffected; new callers pass `format: "html"`.

**Request** (identical body to `/api/render`):

```json
{
  "document_type": "proposal",
  "format": "html",
  "client": "acme",
  "content": "---\ntitle: ...\n---\n\n# Body..."
}
```

**Response:**
- `200 OK`
- `Content-Type: text/html; charset=utf-8`
- `Content-Disposition: inline; filename="document.html"` (or `attachment` from `/api/render`)
- Body: a single self-contained `<!DOCTYPE html>` document with inlined CSS and inlined SVG/PNG logos as data URIs

### Option B — dedicated endpoint

`POST /api/render/html` mirroring `/api/render`. Useful if HTML grows its own options (theme overrides, embedded scripts toggle) that don't make sense for PDF/DOCX.

**Recommendation:** start with Option A. Promote to a dedicated endpoint only when HTML-specific options actually exist.

---

## Interactive features (scoped MVP)

The first cut should ship a small, well-defined set of interactions — not a kitchen sink. Suggested MVP:

| Feature | Behaviour |
|---------|-----------|
| **Sticky ToC** | Auto-generated from H1/H2/H3, fixed left rail on desktop, collapsible drawer on mobile |
| **Anchor links** | Every heading gets a stable `id` (already produced by the parser at `src/parser/markdown.rs`) plus a click-to-copy permalink icon on hover |
| **Code block copy** | Copy-to-clipboard button on every fenced code block, with language label |
| **Collapsible sections** | Any H2 (or H3) renders with a chevron that toggles its body — closed-by-default opt-in via frontmatter or a `collapse:` HTML comment marker |
| **Dark mode** | Toggle in the header; respects `prefers-color-scheme` on first load; persists via `localStorage` |
| **Print stylesheet** | `@media print` block strips chrome (ToC, toggles, dark mode) so users can still print to PDF from the browser |

**Out of scope for v1:**
- Forms / inputs / signature capture
- Live data / WebSocket updates
- Multi-page navigation (everything is one scroll)
- Embedded video or third-party iframes
- User authentication on the rendered output itself

---

## Branding mapping

The existing `ClientProfile` maps to CSS as follows:

| Profile field | CSS / HTML use |
|---------------|----------------|
| `colour_primary` | `--color-primary` (headings, links) |
| `colour_secondary` | `--color-secondary` (sub-headings, captions) |
| `colour_accent` | `--color-accent` (buttons, highlights) |
| `colour_text` | `--color-text` (body) |
| `colour_background` | `--color-bg` |
| `colour_table_header` / `_border` / `_callout_bg` | Direct CSS variables, same names |
| `font_heading` / `font_body` / `font_mono` | `font-family` on `h1–h6`, `body`, `pre/code` |
| `font_size_base` / `_h1` / `_h2` / `_h3` | `font-size` (in `rem` after dividing by 16) |
| `line_height` | `line-height` on body |
| `margin_*` | Page-level `padding` on a `.page` wrapper (converted from mm) |
| `logo_light_url` / `logo_dark_url` | `<img>` in header — swapped based on dark-mode state |
| `cover_enabled` + `cover_template` | Optional `<section class="cover">` at the top |
| `header_enabled` + `header_template` | `<header>` element |
| `footer_enabled` + `footer_template` | `<footer>` element |
| `watermark_text` + `watermark_opacity` | Fixed-position `<div>` behind content with rotated text |

A dark-mode palette would be derived automatically (invert background/text, dim primary by ~15% saturation) unless the profile gains explicit dark-variant fields later.

---

## Example output (sketch)

For an input of:

```markdown
---
title: Engagement Proposal
recipient: Acme Corp
date: 2026-05-12
---

# Executive Summary

We propose a five-day discovery phase.

## Scope

- Workshops
- Interviews
- Findings report
```

The renderer would produce (abbreviated):

```html
<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
  <meta charset="utf-8">
  <title>Engagement Proposal</title>
  <style>
    :root {
      --color-primary: #FF6600;
      --color-text: #1A1A1A;
      --color-bg: #FFFFFF;
      --font-body: Helvetica, sans-serif;
      /* ... */
    }
    [data-theme="dark"] { --color-bg: #111; --color-text: #EEE; /* ... */ }
    /* sticky ToC, code copy buttons, collapsibles, etc. */
  </style>
</head>
<body>
  <header class="doc-header">
    <img src="data:image/png;base64,..." alt="logo">
    <button class="theme-toggle" aria-label="Toggle dark mode">…</button>
  </header>

  <nav class="toc">
    <ol>
      <li><a href="#executive-summary">Executive Summary</a></li>
      <li><a href="#scope">Scope</a></li>
    </ol>
  </nav>

  <main class="page">
    <section class="cover">
      <h1>Engagement Proposal</h1>
      <p class="recipient">Acme Corp</p>
      <p class="date">2026-05-12</p>
    </section>

    <h1 id="executive-summary">
      Executive Summary
      <a class="anchor" href="#executive-summary" aria-label="Permalink">#</a>
    </h1>
    <p>We propose a five-day discovery phase.</p>

    <details open>
      <summary><h2 id="scope">Scope</h2></summary>
      <ul>
        <li>Workshops</li>
        <li>Interviews</li>
        <li>Findings report</li>
      </ul>
    </details>
  </main>

  <footer class="doc-footer">…</footer>
  <script>/* theme toggle, copy buttons, ToC active-section highlighting */</script>
</body>
</html>
```

Everything inlined — no external CSS, no CDN dependencies, no fetches at runtime. The output is a single file you can email, drop in S3, or open offline.

---

## Implementation sketch

Roughly what would change in the codebase:

1. **`src/models/render_job.rs`** — add `Html` to `RenderFormat`.
2. **`src/renderers/html.rs`** (new) — mirrors the shape of `pdf.rs` / `docx.rs`:
   ```rust
   pub fn render(branded: &BrandedDocument) -> anyhow::Result<Vec<u8>>
   ```
   Returns UTF-8 HTML bytes. Uses a small template (could be a plain `format!` to start, swapped for `askama` or `minijinja` if templates grow).
3. **`src/routes/render.rs` & `src/routes/preview.rs`** — add an arm to the `match req.format` block setting `content_type = "text/html; charset=utf-8"` and `ext = "html"`.
4. **No DB changes.** Render history already stores the `format` column as a string — `"html"` slots in.
5. **No new dependencies required.** Inlining logos can use `base64` (already pulled in transitively, or add `base64 = "0.22"`). For HTML escaping, `pulldown-cmark` already offers `push_html`, but we want our own walker over `DocumentNode` to apply branding hooks — so a tiny manual `escape_html` helper is enough.

**Effort estimate:** ~1 day for a solid MVP including the interactive features in the table above. Most of that is CSS, not Rust.

---

## Open questions

These need answers before building, not after:

1. **Caching.** PDFs are heavy and worth caching; HTML is cheaper to regenerate. Do we still write to render history, or treat HTML as ephemeral?
2. **Sanitisation.** The markdown parser is trusted today, but HTML opens an XSS surface if a future "raw HTML passthrough" feature is added. Decide upfront whether raw HTML in markdown is escaped (safe default) or rendered (needs sanitiser like `ammonia`).
3. **Asset embedding.** Logos referenced by `logo_light_url` are currently external URLs. For self-contained HTML we'd need to fetch + inline at render time. Adds latency and a new failure mode — worth it for portability, but worth deciding.
4. **Interactive features as opt-in.** Should `format: "html"` always include the JS, or should it be `format: "html"` (static) + `format: "html-interactive"` (with scripts)? The latter lets callers ship a JS-free HTML to email clients without surprises.
5. **Page size in HTML.** PDF respects A4 / Letter via `page_size`. HTML doesn't have pages — do we honour `page_size` as max-width on the `.page` wrapper, or ignore it?

---

## Out of scope (explicitly)

To keep v1 shippable:

- WYSIWYG editing of the rendered HTML
- Server-rendered React / Vue / Svelte components
- WebComponents / Shadow DOM
- Backwards-compatible HTML for IE11 or other legacy targets
- Server-sent events for live document updates

These are deliberate exclusions — bring them back as separate proposals if a real use case appears.
