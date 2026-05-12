# Typeset API Reference

Typeset turns markdown (with optional YAML frontmatter) into branded PDF or DOCX files. Send markdown in, get a file back.

- **Base URL:** `https://typeset.chrisgarlick.com`
- **Auth:** `Authorization: Bearer <api-key>` on every authenticated endpoint
- **Content type:** `application/json` for all request bodies

---

## Authentication

API keys use the format:

```
ts_<uuid>_<random>
```

The middleware parses the embedded UUID and treats it as your user ID. Every authenticated endpoint scopes data (client profiles, render history) to that UUID, so **always reuse the same key** for the same caller — a different UUID is effectively a different account.

### Generating a key

Pick a UUID v4 (any tool — `uuidgen`, an online generator, etc.) and append a random suffix. Example:

```
ts_550e8400-e29b-41d4-a716-446655440000_prod-key-1
```

Send it as:

```
Authorization: Bearer ts_550e8400-e29b-41d4-a716-446655440000_prod-key-1
```

> Note: the current build trusts the UUID embedded in the key (MVP behaviour). Keep your key private and treat it as a secret. If you want stricter validation later, that's an additive change and won't break existing keys.

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
  "client": "optional client profile slug",
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
- `client` — optional. If omitted, the built-in default profile is used. If supplied, the slug must exist under your API key's user ID.
- `overrides` — any field here overrides the equivalent frontmatter value at render time.

### Response

- **Success (200):** raw file bytes
  - `Content-Type: application/pdf` (or the DOCX mime type)
  - `Content-Disposition: attachment; filename="document.pdf"`
- **Errors:** JSON `{ "error": "...", "status": <code> }`
  - `401` — missing or malformed `Authorization` header
  - `404` — referenced `client` slug doesn't exist for this user
  - `500` — render failure or DB error

### curl

```bash
curl -X POST https://typeset.chrisgarlick.com/api/render \
  -H "Authorization: Bearer ts_550e8400-e29b-41d4-a716-446655440000_prod" \
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
    "Authorization": `Bearer ${API_KEY}`,
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
    headers={"Authorization": f"Bearer {API_KEY}"},
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
| `client` | string | Client profile slug |

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

Inline bold/italic/code work within paragraphs. Page breaks can be inserted via the renderer's logic (look for explicit markers in `examples/` if needed).

---

## Client profiles

A client profile is a saved bundle of branding (colours, fonts, margins, logo, header/footer, cover, watermark). Reference one in a render request via the `client` field.

### GET /api/clients

Returns an array of your profiles.

```bash
curl -H "Authorization: Bearer $API_KEY" \
  https://typeset.chrisgarlick.com/api/clients
```

### POST /api/clients

Create or upsert a profile. The only required fields are `slug` and `name` — every other field has a sensible default.

```bash
curl -X POST https://typeset.chrisgarlick.com/api/clients \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "slug": "acme",
    "name": "Acme Corp",
    "colour_primary": "#FF6600",
    "colour_accent": "#003366",
    "font_heading": "Helvetica",
    "font_body": "Helvetica",
    "page_size": "A4",
    "cover_enabled": true,
    "cover_template": "minimal",
    "logo_light_url": "https://example.com/acme-logo.png",
    "watermark_text": "CONFIDENTIAL"
  }'
```

Optional fields (defaults shown):

| Field | Default |
|-------|---------|
| `colour_primary` | `#000000` |
| `colour_secondary` | `#333333` |
| `colour_accent` | `#0066CC` |
| `colour_text` | `#1A1A1A` |
| `colour_background` | `#FFFFFF` |
| `colour_table_header` | `#F5F5F5` |
| `colour_table_border` | `#E0E0E0` |
| `colour_callout_bg` | `#F8F9FA` |
| `font_heading` | `Helvetica` |
| `font_body` | `Helvetica` |
| `font_mono` | `Courier` |
| `font_size_base` | `11.0` |
| `font_size_h1` | `24.0` |
| `font_size_h2` | `18.0` |
| `font_size_h3` | `14.0` |
| `line_height` | `1.5` |
| `page_size` | `A4` |
| `margin_top` / `bottom` / `left` / `right` | `25.4` (mm) |
| `paragraph_spacing` | `8.0` |
| `section_spacing` | `16.0` |
| `logo_light_url` / `logo_dark_url` | `null` |
| `logo_width` | `40.0` |
| `logo_position` | `left` |
| `cover_enabled` | `false` |
| `cover_template` | `minimal` |
| `cover_bg_colour` | `#000000` |
| `cover_text_colour` | `#FFFFFF` |
| `header_enabled` | `true` |
| `header_template` | `logo-left` |
| `header_border` | `true` |
| `footer_enabled` | `true` |
| `footer_template` | `page-numbers` |
| `footer_border` | `true` |
| `watermark_text` | `null` |
| `watermark_opacity` | `0.08` |

### GET /api/clients/:slug

Returns one profile. `404` if not found for this user.

### DELETE /api/clients/:slug

```bash
curl -X DELETE \
  -H "Authorization: Bearer $API_KEY" \
  https://typeset.chrisgarlick.com/api/clients/acme
```

Returns `{ "deleted": true }`.

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
curl -H "Authorization: Bearer $API_KEY" \
  "https://typeset.chrisgarlick.com/api/history?limit=50&client=acme"
```

---

## Complete end-to-end example

### 1. Mint a key

```
ts_550e8400-e29b-41d4-a716-446655440000_prod
```

### 2. (Optional) Create a client profile

```bash
curl -X POST https://typeset.chrisgarlick.com/api/clients \
  -H "Authorization: Bearer ts_550e8400-e29b-41d4-a716-446655440000_prod" \
  -H "Content-Type: application/json" \
  -d '{"slug":"acme","name":"Acme Corp","colour_primary":"#FF6600"}'
```

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
  -H "Authorization: Bearer ts_550e8400-e29b-41d4-a716-446655440000_prod" \
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
| `404` | Client profile slug not found for this user |
| `500` | Render error, DB error, or capacity exceeded |

---

## Operational notes

- **Concurrency:** the server caps simultaneous renders via a semaphore. Excess calls queue rather than fail, but very heavy bursts can return `500 "Render capacity exceeded"`. Add retry-with-backoff in your client if you fan out widely.
- **Timeouts:** keep client timeouts ≥ 30s for large documents.
- **Idempotency:** the API is not idempotent — every call produces a new render and (for `/api/render`) a new history entry. Deduplicate client-side if needed.
- **CORS:** wide-open (`Access-Control-Allow-Origin: *`). Safe to call from browsers, but treat keys accordingly.
