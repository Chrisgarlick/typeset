# Typeset API Reference

Typeset turns markdown (with optional YAML frontmatter) into branded PDF or DOCX files. Send markdown in, get a file back.

- **Base URL:** `https://typeset.chrisgarlick.com`
- **Auth:** `Authorization: Bearer <token>` on every authenticated endpoint
- **Content type:** `application/json` for all request bodies

---

## Authentication

Every authenticated request sends a **caller key from Clients** (https://clients.kritano.com):

```
Authorization: Bearer ck_…
```

Keys are created and revoked in Clients (`clients-admin keys create <name> --kind caller [--clients a,b]`). A key limited to some clients can only render with those clients' brands (`403` otherwise). Typeset checks a key with Clients and remembers the answer for 60 seconds, so a revoked key stops working within a minute.

**Transition:** while `TYPESET_TOKEN` is set on the server, the old shared secret is still accepted (with access to every client). Unset it once every caller has a Clients key.

Treat keys like passwords: never commit them, never ship them to browsers you don't trust, and revoke and replace one in Clients if it leaks.

---

## Rate limiting

`/api/render` and `/api/preview` are limited to **10 requests per minute per IP** by default. Beyond that limit you'll get:

```
HTTP 429 Too Many Requests
Retry-After: <seconds>

{ "error": "Rate limit exceeded — try again in 38s", "status": 429 }
```

Configure via the `TYPESET_RATE_LIMIT_PER_MIN` env var. The limit only applies to the rendering endpoints — `/api/clients` and `/api/history` are unlimited (still behind the bearer token).

The limiter respects `X-Forwarded-For` and `X-Real-IP`, so it works correctly behind nginx or any reverse proxy.

---

## Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| GET    | `/health` | No | Liveness check |
| GET    | `/api/health` | No | Liveness check |
| POST   | `/api/render` | Yes | Render markdown → PDF / DOCX (download) |
| POST   | `/api/preview` | Yes | Render markdown → PDF / DOCX (inline) |
| GET    | `/api/clients` | Yes | List your client profiles |
| POST   | `/api/clients` | Yes | Create or update a client profile |
| GET    | `/api/clients/:slug` | Yes | Get one client profile |
| DELETE | `/api/clients/:slug` | Yes | Delete a client profile |
| GET    | `/api/history` | Yes | List previous renders |

---

## POST /api/render

The main endpoint. Returns the rendered file as a download (`Content-Disposition: attachment`).

### Request body

```json
{
  "content": "string — full markdown, with YAML frontmatter at the top",
  "document_type": "proposal | report | brief | sop | invoice | general",
  "format": "pdf | docx | both",
  "client": "optional: the client's slug in Clients",
  "brand_version": "optional: a published brand version (default: the latest)",
  "input_format": "markdown | json (default markdown)",
  "print": "optional boolean (default false)",
  "overrides": {
    "title": "optional",
    "subtitle": "optional",
    "recipient": "optional",
    "date": "optional",
    "author": "optional"
  }
}
```

Field notes:
- `content` — the entire markdown document as one string. YAML frontmatter goes **inside** this string, between `---` fences at the top (see example below).
- `document_type` — currently used for logging/categorisation. Must be one of the values listed.
- `format` — `"pdf"`, `"docx"`, or `"both"`. (`"both"` currently returns the PDF only.)
- `client` — optional. The client's slug in Clients; its published brand (palette, fonts, logos, print settings) styles the document. If omitted, the frontmatter's `client`, else Typeset's default look. An unknown client is `404`; a client the key may not use is `403`.
- `brand_version` — optional. Pin to a published brand version (for re-rendering an old document exactly); the latest when omitted.
- `input_format` — `"markdown"` (the default) or `"json"`, a structured document; see [JSON documents](#json-documents).
- `print` — `true` for professional printing: every colour in CMYK and a 3mm bleed on each edge of every page (an A4 document comes out 216 × 303mm, to be trimmed to A4). Full-page fills, such as the cover, run into the bleed. Leave it off for screen and office printing.

PDFs are tagged PDF/UA-1, so screen readers can follow the structure: the cover title is the one level 1 heading, sections and subsections follow without skipping levels, and figures carry alt text. A document that can't meet the standard (a version 1 image without alt text, say) is still rendered, as a plain PDF.
- `overrides` — any field here overrides the equivalent frontmatter value at render time.

### Response

- **Success (200):** raw file bytes
  - `Content-Type: application/pdf` (or the DOCX mime type)
  - `Content-Disposition: attachment; filename="document.pdf"`
- **Errors:** JSON `{ "error": "...", "status": <code> }`
  - `400` — the document doesn't parse. For JSON documents the message names the block, e.g. `pages[1].blocks[2] (findings): unknown variant `high`, expected one of `critical`, `serious`, `moderate`, `minor``
  - `401` — missing or malformed `Authorization` header
  - `404` — referenced `client` slug doesn't exist for this user
  - `500` — render failure or DB error

### curl

```bash
curl -X POST https://typeset.chrisgarlick.com/api/render \
  -H "Authorization: Bearer $TYPESET_TOKEN" \
  -H "Content-Type: application/json" \
  -d @payload.json \
  -o output.pdf
```

Where `payload.json` is:

```json
{
  "document_type": "proposal",
  "format": "pdf",
  "content": "---\ntitle: Engagement Proposal\nsubtitle: Q2 2026 Discovery\nrecipient: Acme Corp\ndate: 2026-05-12\nauthor: Chris Garlick\n---\n\n# Executive Summary\n\nAcme has asked us to scope a discovery phase covering...\n\n## Scope\n\n- Workshop facilitation\n- Stakeholder interviews\n- Findings report\n\n## Pricing\n\n| Phase | Days | Total |\n|-------|------|-------|\n| Discovery | 5 | £6,000 |\n| Report | 2 | £2,400 |\n"
}
```

### JavaScript (fetch)

```js
const res = await fetch("https://typeset.chrisgarlick.com/api/render", {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${TYPESET_TOKEN}`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    document_type: "proposal",
    format: "pdf",
    content: markdownString, // see "Building the content string" below
  }),
});

if (!res.ok) throw new Error(await res.text());
const pdfBuffer = await res.arrayBuffer();
// Save to disk, stream to client, etc.
```

### Python (requests)

```python
import requests

resp = requests.post(
    "https://typeset.chrisgarlick.com/api/render",
    headers={"Authorization": f"Bearer {TYPESET_TOKEN}"},
    json={
        "document_type": "proposal",
        "format": "pdf",
        "content": markdown_string,
    },
    timeout=60,
)
resp.raise_for_status()
with open("output.pdf", "wb") as f:
    f.write(resp.content)
```

---

## POST /api/proof

Thumbnails of every page, for checking a layout before rendering the real file. Same body as `/api/render`. Returns JSON:

```json
{ "pages": 9, "ppi": 40, "images": ["<base64 PNG>", "…"] }
```

Rate-limited with the render endpoints.

## POST /api/preview

Identical request body to `/api/render`. The only difference is `Content-Disposition: inline` — the browser displays the file rather than downloading it. Useful for `<iframe src>` previews. **Preview renders are not logged in history.**

---

## YAML frontmatter

Frontmatter is a YAML block at the very top of the markdown content, fenced by `---`:

```markdown
---
title: Engagement Proposal
subtitle: Q2 2026 Discovery
recipient: Acme Corp
date: 2026-05-12
author: Chris Garlick
document_type: proposal
client: acme
---

# Markdown body starts here
```

### Supported frontmatter fields

| Key | Type | Purpose |
|-----|------|---------|
| `title` | string | Document title (used in cover / header) |
| `subtitle` | string | Subtitle / tagline |
| `recipient` | string | Who the document is for |
| `date` | string | Free-form date string |
| `author` | string | Author name |
| `document_type` | string | Same enum as the API field |
| `client` | string | The client's slug in Clients (the request's `client` wins) |
| `mode` | string | A palette from the brand's `modes`, e.g. `dark` (`light` is the base palette) |
| `eyebrow` | string | A short label above the cover title, e.g. "Website review" |
| `reference` | string | A document reference, shown on the cover |
| `cover` | string | Cover style: `editorial`, `band` or `dark`. Defaults to the brand's `print.cover` |

Any other YAML keys are silently ignored. The parser is intentionally simple — single-line `key: value` pairs only. **No nested structures, no arrays, no multi-line strings.** Strip outer quotes (`"` or `'`) are tolerated.

### Building the `content` string

The endpoint expects one string. If your platform has markdown and YAML separately:

```js
const content = `---\n${yaml}\n---\n\n${markdown}`;
```

```python
content = f"---\n{yaml}\n---\n\n{markdown}"
```

---

## Supported markdown

The renderer handles the following CommonMark + GFM elements:

- Headings (H1–H6)
- Paragraphs
- Bullet and ordered lists
- Tables (GFM pipe syntax)
- Code blocks (fenced, with optional language)
- Blockquotes
- Horizontal rules (`---` outside frontmatter)
- Images (`![alt](url)`)
- Strikethrough (`~~text~~`)

Inline bold, italic, code, strikethrough and links work in paragraphs, list items, table cells and headings. A line containing only `{{pagebreak}}` forces a page break; otherwise the renderer places breaks (see the README's authoring rules).

---

## JSON documents

With `"input_format": "json"`, `content` is a JSON document (as a string):

```json
{
  "schema": 2,
  "frontmatter": { "title": "…", "subtitle": "…", "eyebrow": "Website review", "recipient": "…", "author": "…", "date": "…", "cover": "editorial" },
  "pages": [
    { "blocks": [ { "type": "opener", "title": "Summary" }, { "type": "paragraph", "text": "…" } ] }
  ]
}
```

Each entry in `pages` is a section boundary: it starts a new page only when the current one is mostly used.

**Version 1 blocks** (no `schema`, or `"schema": 1`): `heading`, `paragraph`, `markdown`, `bullet_list`, `ordered_list`, `table`, `code`, `blockquote`, `image`, `rule`, `page_break`, `columns` and `section`. These keep working.

- `image` takes `url`, `alt` (required in schema 2 documents), and optionally `caption` and `width` (`column`, the default, or `full`). `url` is an HTTPS address, fetched before rendering, or a base64 data URI (`data:image/jpeg;base64,...`) for images that shouldn't be public. PNG, JPEG, WebP or SVG, up to 5 MB each, at most 24 per document; a whole request can be up to 25 MB. An image that can't be used shows a placeholder with its alt text.
- `section` and `columns` styles accept brand token names instead of raw values: colours `ink`, `ink-muted`, `heading`, `paper`, `surface`, `rule`, `accent`, `accent-soft`, `cover-bg`, `cover-text`, `good`, `fair`, `poor`; lengths `s1` to `s9`, `r-sm`, `r-md`, `hairline`, `heavy`. Hex colours and raw lengths still work.

**Version 2 components** need `"schema": 2`. They say what something is; the theme decides how it looks. They reject unknown fields, and text fields accept inline markdown.

| Type | Fields | Renders as |
|------|--------|------------|
| `opener` | `title`, `eyebrow`?, `intro`?, `stat`? `{ value, label?, status? }` | A numbered section opening (counts as an H2 section), with an optional headline figure |
| `callout` | `body`, `title`?, `tone`? (`info` \| `good` \| `warn` \| `critical` \| `brand`) | A tinted note with a rule in the tone's colour |
| `findings` | `items: [{ title, severity, count?, pages?, fix? }]`, severity `critical` \| `serious` \| `moderate` \| `minor` | A findings table with severity badges and page chips |
| `steps` | `groups: [{ title?, items: [{ title, detail?, effort? }] }]`, or `items` for one group | Numbered steps with effort chips |
| `scores` | `items: [{ label, value, max? (100), benchmark? }]`, `caption`? | Two to five score gauges |
| `metrics` | `items: [{ value, label, target?, status? }]` | Headline figures with a status rule |
| `facts` | `items: [{ label, value }]` | A label and value list |
| `score_matrix` | `columns`, `rows: [{ label, values }]` (a `null` value shows a dash), `max`?, `caption`? | Scores by page as a heat table of tinted pills |
| `bars` | `items: [{ label, value }]`, `unit`?, `target`?, `better`? (`lower` \| `higher`), `caption`? | Horizontal bars with a dashed target line, green within the target and red past it |
| `chart` | `chart` (`line` \| `area` \| `bar` \| `donut` \| `scatter` \| `histogram` \| `heatmap` \| `funnel` \| `bullet`), `labels`, `series: [{ name, values }]` (a `null` value leaves a gap), `title`?, `caption`?, `format`? (`number` \| `compact` \| `percent` \| `currency:GBP`), `stacked`?, `values`? (print bar values), `center`? (`first`: a donut as a score gauge), `bins`?, `height`?, `wide`? | A chart in the brand's colours and fonts, drawn by kit's renderer (the same charts as the web), with its generated summary as alt text. Scatter takes x and y as two series; bullet takes value and target; heatmap rows are labels and columns are series |
| `severity_summary` | `counts: { critical, serious, moderate, minor }`, `caption`? | A segmented bar over four counts |
| `finding` | `title`, `severity`, `why`?, `fix`?, `pages`?, `evidence`? `{ value, label? }` | One headline issue as a card |
| `highlights` | `items`: one to four findings (as `finding`, without `type`) | Headline issues side by side as compact cards |
| `priority_matrix` | `items: [{ label, impact, effort }]`, each `high` \| `low` | Quick wins, big bets, fill-ins and maybe later, with numbered dots and a key |
| `contents` | `title`? | The sections with numbers and page numbers, linked, on a page of its own |
| `back_page` | `about`?, `contact`? `{ name?, email?, website?, phone? }` | A closing page on the cover colour |
| `page` | `url`, `alt` | A whole page that is one image (SVG or PNG), edge to edge and into the bleed in print: a page designed elsewhere, such as Design's A4 frames. It isn't numbered and has no header or footer; it counts towards the 24 images. A document that starts with one gets no generated cover (the designed page is the cover; `title` still names the PDF) |
| `part` | `title`, `eyebrow`?, `intro`? | A full-page divider starting a part of a combined document (a review, then a proposal). Sections after it belong to the part, section numbers run on, and the contents page groups sections under their parts |

`status` is `good`, `fair`, `poor` or `neutral`. The full JSON Schema is [`schema/v2.json`](./schema/v2.json), and [`examples/gallery.json`](./examples/gallery.json) uses every component except `chart`, which [`examples/charts.json`](./examples/charts.json) shows.

In Markdown, a fenced block marked `chart` draws the same chart, with options on the fence line and the data as a table (first column the labels, each other column a series); a JSON body in the fence also works:

````markdown
```chart type=line format=compact title="Score over time"
| Month | Score |
|---|---|
| Jan | 82 |
| Feb | 88 |
```
````

---

## Clients and brands

Brands are managed in **Clients** (https://clients.kritano.com), not in Typeset. Typeset reads a client's published brand when it renders and maps it onto its themes:

| Brand | Typeset |
|-------|---------|
| `palette` (`paper`, `ink`, `ink-muted`, `heading`, `accent`, `surface`, `rule`, `accent-soft`, `inverse`, `on-inverse`) | The theme's colours (cover from `inverse`/`on-inverse`); a frontmatter `mode` picks one of `modes` |
| `type.heading` / `body` / `mono` | Heading, body and code fonts. Library fonts (`lib:…`) are used as uploaded; bundled names (Inter, Inter Tight, JetBrains Mono) as before |
| `type.body.leading` | Line height |
| `logos.primary` / `on-dark` | The logo on light pages / dark covers |
| `wordmark` | The running head when there's no logo |
| `print` (`page`, `margins`, `cover`, `header`, `footer`, `watermark`, `size`) | Page, margins, cover style, header/footer, sizes |

### GET /api/clients

The clients this key may use: `[{ "slug": "kritano", "name": "Kritano", "brand_version": 2 }]`.

### GET /api/clients/:slug

One client's brand as Typeset will use it (`?mode=dark` for a mode): colours, fonts, which logos it has, and the brand version.

### POST /api/clients, DELETE /api/clients/:slug

`410 Gone`: edit brands in Clients.

---

## GET /api/history

Lists past renders for the authenticated user. Preview calls are not included.

Query parameters (all optional):

| Param | Default | Notes |
|-------|---------|-------|
| `limit` | `20` | Max `100` |
| `offset` | `0` | For pagination |
| `client` | — | Filter by client slug |
| `format` | — | `pdf` / `docx` / `both` |

```bash
curl -H "Authorization: Bearer $TYPESET_TOKEN" \
  "https://typeset.chrisgarlick.com/api/history?limit=50&client=acme"
```

---

## Complete end-to-end example

### 1. Get a caller key

Create one in Clients (on the Kritano droplet) and export it locally so the examples below work:

```bash
clients-admin keys create my-app --kind caller --clients acme   # prints ck_…
export TYPESET_TOKEN="ck_…"
```

### 2. Make sure the client has a published brand

In https://clients.kritano.com: create the client (`acme`), upload its fonts and logos, paste the brand, Validate, Publish.

### 3. Build the markdown + YAML

`document.md`:

```markdown
---
title: Engagement Proposal
subtitle: Q2 2026 Discovery
recipient: Acme Corp
date: 2026-05-12
author: Chris Garlick
---

# Executive Summary

Acme has asked us to scope a five-day discovery phase covering stakeholder
interviews, workshop facilitation, and a final findings report.

## Scope

- Three half-day workshops
- Six stakeholder interviews
- Written findings report (~15 pages)

## Pricing

| Phase     | Days | Total   |
|-----------|------|---------|
| Discovery | 5    | £6,000  |
| Report    | 2    | £2,400  |
| **Total** |      | **£8,400** |

## Next Steps

1. Sign and return this proposal
2. Kick-off call within 5 working days
3. Workshops scheduled across weeks 2–3
```

### 4. Render

```bash
curl -X POST https://typeset.chrisgarlick.com/api/render \
  -H "Authorization: Bearer $TYPESET_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$(jq -Rs '{
    document_type: "proposal",
    format: "pdf",
    client: "acme",
    content: .
  }' document.md)" \
  -o proposal.pdf
```

You now have `proposal.pdf` on disk.

---

## Error reference

All errors return JSON:

```json
{ "error": "human-readable message", "status": 404 }
```

| Status | Meaning |
|--------|---------|
| `400` | Bad request — malformed JSON or invalid enum value |
| `401` | Missing or malformed `Authorization` header |
| `403` | The key can't use this client (limited to others in Clients) |
| `404` | No such client in Clients, or no such brand version |
| `410` | `POST`/`DELETE /api/clients`: brands are edited in Clients now |
| `503` | Clients can't be reached and the brand isn't cached, or Typeset isn't connected to Clients |
| `500` | Render error, DB error, or capacity exceeded |

---

## Operational notes

- **Concurrency:** the server caps simultaneous renders via a semaphore. Excess calls queue rather than fail, but very heavy bursts can return `500 "Render capacity exceeded"`. Add retry-with-backoff in your client if you fan out widely.
- **Timeouts:** keep client timeouts ≥ 30s for large documents.
- **Idempotency:** the API is not idempotent — every call produces a new render and (for `/api/render`) a new history entry. Deduplicate client-side if needed.
- **CORS:** wide-open (`Access-Control-Allow-Origin: *`). Safe to call from browsers, but treat keys accordingly.
