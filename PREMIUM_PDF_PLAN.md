# Typeset: from good to premium (10/100 to 100/100)

A plan to make every PDF Typeset produces look and feel premium: considered typography, a real
component library, data shown as visuals rather than text, and print-quality polish. Written on
20 October 2026 from the renderer (`src/renderers/typst.rs`), the live branding profiles, and a
page-by-page critique of a real report (the Myers Jewellers website review from `scout review`).

---

## 1. Where it is today: an honest audit

### 1.1 Things that are actually broken (fix first)

| # | Problem | Evidence | Effect |
|---|---|---|---|
| B1 | **Every PDF is set in Libertinus Serif**, Typst's built-in fallback | `pdffonts` on a live report shows only LibertinusSerif. `typst fonts` in the container lists only Typst's three embedded fonts (DejaVu Sans Mono, Libertinus Serif, New Computer Modern). Inter is installed in `/usr/share/fonts/opentype/inter`, but Typst doesn't see system fonts. | The brand fonts never apply. This is the single biggest reason the PDFs look "nearly right". |
| B2 | **Headings ignore `font_heading`** | `write_preamble` sets only the body font; `p.font_heading` is never read; `font_mono` is read and discarded (`let _ = &p.font_mono`). | No typographic contrast between headings and text. |
| B3 | **Logos never render** | `chris-garlick-light` has `logo_light_url` set; the renderer has no logo code. | Unbranded covers and headers. |
| B4 | **Images are placeholders** | `DocumentNode::Image` emits `[image: url]` text. | No screenshots, no illustrations. |
| B5 | **No inline formatting** | `typst_inline` is `typst_content` (escapes everything); the markdown parser drops Strong, Emphasis and Link. | No bold, italics or links anywhere. Emphasis has to be faked with headings. |
| B6 | **Profiles out of date** | `chris-garlick-light`: Instrument Serif headings, **DM Mono body text** (from the old brand). | Even with fonts fixed, it would render the old identity, with body copy in a monospace font. |
| B7 | `cover_enabled`, `cover_template`, `header_template`, `footer_template`, `watermark_*` | Stored on profiles, unused by the Typst renderer. | Settings that do nothing. |
| B8 | Typst 0.13.0 | `typst --version` in the container. | Missing 0.14's tagged PDF and accessibility output, and recent layout fixes. |

### 1.2 Design critique of real output (Myers Jewellers review, 14 pages)

- **Cover:** a large black field with the title floating in the middle, a thin accent rule, a
  fallback serif, and tiny all-caps metadata labels (8pt) at the bottom. There's no logo, no
  brand device, no hierarchy between the client and the document type, and the title wraps
  awkwardly ("Website review: / myersjewellers.co.uk").
- **Headings:** 26pt H1 and 18pt H2 in the body font, bold. They feel like a word processor's
  defaults rather than an editorial system. No section numbers, no eyebrow labels.
- **Pagination:** every H1 and H2 forces a page break, so sections that are half a page long leave
  half-empty pages (Next steps, Other things I noticed).
- **Tables carry most of the content and look the weakest:**
  - full grid borders
  - a beige header row
  - severity as plain words ("Critical", "Serious") rather than colour-coded badges
  - four columns of nearly equal width, so the "What to do" text wraps into tall cells
  - no zebra or row rhythm
  - repeated phrases ("All 5 pages") with no visual shorthand
- **Score cards:** flat pastel boxes with "36 / 100" in body text. No gauge, no benchmark, and
  no sense of good, fair or poor beyond the background tint.
- **Next steps:** two plain numbered lists. They read as an appendix, not as the plan the reader is
  being sold.
- **No contents page**, no executive one-pager, and no "at a glance" summary.
- **The header and footer** are a tiny document title and "Page X of Y" separated by hairlines.
  It's functional but anonymous.
- **Overall:** about 10/100 for premium feel. The content is strong; the presentation is generic.

---

## 2. What 100/100 looks like

A reader should feel the document was designed, not generated:

1. **Typography with a point of view:** a display face for headings (Inter Tight) and a
   text face for reading (Inter), a modular type scale, real small caps or tracked labels,
   optical sizes, tabular figures in tables and scores, proper quotes and dashes, hyphenation
   tuned, no widows or orphans.
2. **A grid:** a 12-column page grid with a generous outer margin. Every component snaps to
   it, and the rhythm is consistent (an 8pt baseline-ish spacing scale).
3. **Components, not formatting:** the document is assembled from a library of designed
   components (cover, section opener, score gauge, finding card, step plan, callout, KPI row,
   contents). Authors (people or Claude) choose components; the design system decides how they
   look.
4. **Data as visuals:** scores as gauges or bars, severity as colour-coded badges with counts,
   priorities as a two-by-two of impact against effort, before and after as paired metrics.
5. **Brand in every page:** the logo, brand colours used as accents (never as large flat fills that
   fight the text), a recognisable cover, and running heads with section names.
6. **Print-perfect polish:** no stranded headings, tables that split cleanly with repeated
   headers, sensible page breaks, PDF metadata, bookmarks and tagged PDF output.

**The bar:** put a Typeset report next to a report from a top consultancy or design studio, and it
holds up.

---

## 3. The design system

### 3.1 Tokens (one source of truth)

Every component reads tokens; nothing hard-codes a colour or size. Tokens come from the client
profile, with sensible defaults, and are emitted once as a Typst dictionary (`#let t = (...)`).

**Colour roles.** The current profile fields are mapped to roles; new fields are added where
needed.

| Role | Purpose | chris-garlick-light (new) |
|---|---|---|
| `ink` | text, headings | `#151513` |
| `ink-muted` | captions, meta | `#5B5852` |
| `paper` | page background | `#FFFFFF` (or `#FAF8F3` for an off-white "paper" theme) |
| `surface` | cards, table header | `#F2EEE6` |
| `rule` | lines, borders | `#DAD5CA` |
| `accent` | brand accent: rules, numbers, highlights | `#1F5C45` |
| `accent-soft` | tints behind accent content | `#DDE7DF` |
| `good` / `fair` / `poor` | status scale (scores) | `#1F7A4D` / `#B7791F` / `#B42318` |
| `critical` / `serious` / `moderate` / `minor` | severity | from the status scale, plus a neutral for minor |

All status colours must pass WCAG AA on `paper`, and their tints must pass with `ink`. Add a
test that checks contrast for every profile.

**Type scale.** A modular scale at a 1.25 ratio from an 11pt base, set as tokens:

| Token | Size / leading | Face | Use |
|---|---|---|---|
| `display` | 44 / 46 | Inter Tight 600, -2% tracking | Cover title |
| `h1` | 30 / 34 | Inter Tight 600, -1.5% | Section opener |
| `h2` | 20 / 26 | Inter Tight 600, -1% | Subsection |
| `h3` | 14 / 19 | Inter Tight 600 | Card titles |
| `eyebrow` | 8.5 / 12 | Inter 600, +12% tracking, uppercase | Labels above headings |
| `body` | 10.5 / 16 | Inter 400 | Reading text |
| `small` | 9 / 13 | Inter 400 | Table cells, captions |
| `numeral` | 28 to 56 | Inter Tight 600, tabular figures | Scores, KPIs |

**Spacing:** `2, 4, 8, 12, 16, 24, 32, 48, 64` pt as `s1` to `s9`. **Radius:** `r-sm` 3pt,
`r-md` 6pt. **Rules:** hairline 0.5pt `rule`; heavy 2.5pt `ink` (for section openers, echoing
the site's double rule).

### 3.2 Fonts (fixes B1, B2, B6)

- Bundle the font files in the repo: `assets/fonts/` holds Inter (variable), Inter Tight (variable
  plus italic), IBM Plex Mono or JetBrains Mono for code, and one serif option (Newsreader or
  Source Serif 4). All are OFL.
- Compile with `typst compile --font-path assets/fonts --ignore-system-fonts`, so output is
  identical on every machine and container, and a missing font fails loudly instead of silently
  falling back.
- A font registry maps profile font names to bundled families. An unknown name gets a validation
  error at profile save, not a silent fallback at render.
- Turn on the OpenType features that matter: `tnum` in tables and scores, `ss01` and `cv11` where
  the face has them, proper quotes (`smartquote`), and `hyphenate: true` with `lang: "en"` and
  `region: "gb"`.
- Add a test that renders a sample and asserts with `pdffonts` that only the expected families are
  embedded.

### 3.3 Page system

- **A4 with a 12-column grid:** 20mm inner margin, 24mm outer, 22mm top and bottom.
- **The body text block is 8 of 12 columns** wide on body pages, with a margin column for eyebrow
  labels, section numbers and side notes. Wide components (tables, score grids) span all 12.
- **Running head:** the logo mark (small) on the left, the current section name on the right
  (from `query(heading)`), and a hairline below.
- **Footer:** document title or client on the left, "Page 4 of 13" on the right, tabular figures.
- **Page breaks:** H1 (section openers) start a new page. H2 never forces a break. Instead,
  headings are `sticky` and blocks use `breakable: false` where splitting would look wrong, so
  half-empty pages disappear.
- **Widows and orphans:** set Typst's paragraph and widow settings, and keep each heading with
  at least two lines of what follows.
- **Bookmarks:** section headings become PDF outline entries.

---

## 4. The component library

Each component has three parts:
- a **JSON block type**, which is what authors and Claude write
- a **Typst function** in a bundled `theme.typ`, which is how it looks
- a **spec**: the visual rules it has to meet

Components live in `src/renderers/theme/` as `.typ` files, compiled together with the document, so
designers can work on them as plain Typst with live preview.

### 4.1 Document furniture

| Component | JSON | Visual spec |
|---|---|---|
| **Cover** | `frontmatter` plus `cover: { style: "editorial" \| "band" \| "minimal", eyebrow, image? }` | Editorial: paper background with the logo at top left, an eyebrow ("Website review"), the client as the display title, a subtitle, then a hairline grid of metadata (Prepared for, Prepared by, Date, Reference) at the bottom in two columns. A brand device: a heavy accent rule or a large tinted numeral. Band: the same, with a 40% accent band at the top. No dead centre. |
| **Contents** | `{ "type": "contents" }` | Section numbers in accent, titles in `h3`, dotted leaders, page numbers in tabular figures. Built from the document's H1s. |
| **Section opener** | `{ "type": "section", "number": "02", "title": "Accessibility", "eyebrow": "Findings", "intro": "..." }` | The large section number in accent `numeral`, a heavy rule, the title in `h1`, and an intro paragraph at 12.5pt. Optionally a key stat on the right (e.g. "36 / 100" plus a status pill). Starts a page. |
| **Running head and footer** | automatic | As in section 3.3. |
| **Back page** | `{ "type": "back_page", "contact": {...} }` | Brand mark, contact details and a short "About" line. |

### 4.2 Data components

| Component | JSON | Visual spec |
|---|---|---|
| **Score gauge** | `{ "type": "score", "label": "Accessibility", "value": 36, "max": 100, "benchmark": 90 }` | A 270° arc drawn in Typst (`curve`/`circle`), coloured by status, with the value centred in a `numeral` and the label below. Good, fair and poor thresholds come from tokens. |
| **Score row** | `{ "type": "scores", "items": [...] , "caption": "Average across 5 pages" }` | Two to five gauges on the grid, with one caption line. Replaces today's pastel boxes. |
| **Score table by page** | `{ "type": "score_matrix", "columns": [...], "rows": [{ "label": "/shop/", "values": [35, 42, 61, 57] }] }` | A heat table: each cell a tinted pill with a tabular figure, coloured by status; the label column in `small`. |
| **KPI / metric** | `{ "type": "metrics", "items": [{ "value": "7.7 MB", "label": "Home page size", "target": "< 0.5 MB", "status": "poor" }] }` | A large value, the label, then the target in muted text, with a status bar underneath. Two to four per row. |
| **Bar comparison** | `{ "type": "bars", "items": [{ "label": "Home", "value": 15.0, "unit": "s" }], "target": 3 }` | Horizontal bars with a dashed target line, for load times and page sizes across pages. |
| **Severity summary** | `{ "type": "severity_summary", "counts": { "critical": 2, "serious": 9, ... } }` | A segmented bar plus four count chips. |

### 4.3 Findings and actions

| Component | JSON | Visual spec |
|---|---|---|
| **Severity badge** | inline, or a field on findings | A pill with a coloured dot and the severity label in tracked small caps. Critical is filled; the others are outlined or tinted. |
| **Findings table** | `{ "type": "findings", "items": [{ "title", "severity", "count", "pages": [...], "fix" }] }` | Columns sized by content (`auto, 1fr, 2fr`); severity as a badge; the count as a tabular figure; pages as small chips ("All 5" collapses to one chip); the fix in `small`. No vertical rules, hairline row separators, zebra on `surface`, the header row in `eyebrow` style, and a repeated header when it splits across pages. |
| **Finding card** | `{ "type": "finding", "title", "severity", "why", "fix", "pages", "evidence"? }` | For the headline issues: a card with a severity bar on the left, "Why it matters" and "What to do" as two columns, and evidence (a measured value) as a metric chip. |
| **Priority matrix** | `{ "type": "priority_matrix", "items": [{ "label", "impact": "high", "effort": "low" }] }` | A two-by-two (quick wins, big bets, fill-ins, maybe later) with numbered dots and a key list. |
| **Step plan** | `{ "type": "steps", "groups": [{ "title": "Quick wins", "items": [{ "title", "detail", "effort": "1 hour" }] }] }` | Numbered steps with accent numerals, a title and detail, an effort chip on the right, and a group header with a hairline. Replaces plain numbered lists for "Next steps". |
| **Checklist** | `{ "type": "checklist", "items": [...] }` | Empty boxes, so it works as a printable to-do list. |

### 4.4 Text components

| Component | JSON | Visual spec |
|---|---|---|
| **Callout** | `{ "type": "callout", "tone": "info" \| "good" \| "warn" \| "critical" \| "brand", "title", "body" }` | A tinted surface with a 3pt left rule in the tone colour, an icon glyph, the title in `h3` and the body in `body`. Never a dark fill (which hides headings: see the `scout review` template notes). |
| **Pull quote** | `{ "type": "quote", "text", "attribution" }` | `h2` size, Inter Tight italic, with accent quotation marks hung in the margin. |
| **Key-value list** | `{ "type": "facts", "items": [{ "label", "value" }] }` | Two-column definition list with labels in `eyebrow`. |
| **Paragraph / markdown** | as today, **with inline formatting** | Bold, italic, links (accent, underlined on print), inline code. Fixes B5. |
| **Figure** | `{ "type": "image", "src", "caption", "width": "full" \| "column" }` | Fixes B4: images fetched before compile (with a size limit, https only, and an allowlist or a data URI), written into the temp root, a caption in `small`, an optional hairline frame. |
| **Appendix table** | `{ "type": "reference_table", ... }` | Denser: `small` size, monospaced references, no zebra. |

### 4.5 Layout primitives (keep, make token-aware)

`columns` and `section` stay, but their styling options use **token names** (`"surface"`,
`"accent-soft"`, `"r-md"`, `"s5"`) instead of raw hex and lengths, so authors can't produce
off-brand colours. Raw values stay accepted for backward compatibility, behind a
`strict_tokens` flag.

---

## 5. Brand profiles (fixes B3, B6, B7)

- Update `chris-garlick-light` to the current brand: Inter Tight and Inter, ink `#151513`,
  accent `#1F5C45`, a paper or white option, and the "CG" logo mark. Retire or rename
  `chris_garlick_dark` (a duplicate of `chris-garlick-dark`).
- Add profile fields: `font_display`, `theme` (`editorial` | `band` | `minimal`),
  `paper_tint`, the status colours, `logo_mark_url` (a square mark for running heads).
- Logos are fetched at render (cached by URL and checksum). SVG is preferred; PNG at 3x or more.
  Logo render code lands, and the `cover_*`, `header_*` and `footer_*` settings are honoured or
  removed.
- **Profile validation:** contrast checks, known fonts, reachable logo. Return errors on save.

---

## 6. Schema and authoring

- **Semantic blocks over styling.** The JSON gains the component types above. Authors say
  *what* something is (a finding, a score, a step), never how it looks. The current
  `section`, `columns`, `heading` and `table` blocks keep working.
- **Version the schema:** `"schema": 2` at the top. Version 1 documents render exactly as now.
  Version 2 unlocks components and inline formatting.
- **Validation:** a JSON Schema file (`schema/v2.json`) used by the API (returning 400 with exact
  paths), by `scout` (before sending), and published in the docs. Unknown fields are rejected in
  v2.
- **A component gallery document** (`examples/gallery.json`) that uses every component. It's the
  visual spec, the regression fixture and the reference for prompt authors.
- **Update `scout`'s `templates/typeset-format.md`** and its validator to the v2 components, so
  Claude writes `score_matrix`, `findings`, `steps` and `metrics` instead of hand-built columns
  and tables.

---

## 7. Quality and tooling

- **Visual regression:** render `examples/gallery.json` and each example to PNG
  (`typst compile --format png --ppi 110`) in CI, then compare against approved snapshots with a
  small pixel tolerance. Diffs show as images in the PR.
- **A "proof" endpoint:** `POST /api/proof` returns PNG thumbnails of every page, for quick
  review in the frontend and for `scout` to check a draft.
- **Lint the output:**
  - no Libertinus or New Computer Modern embedded
  - page count within a range
  - no page under 15% full except openers and the back page
  - every heading followed by content on the same page
- **Accessibility:**
  - Typst 0.14 tagged PDF (PDF/UA-1 where possible)
  - a document title and language in metadata
  - alt text required on figures
  - reading order matches visual order
  - contrast checked for every token pair
- **Performance:** render under 2 seconds for a 15-page report; fonts and theme cached. Keep the
  current concurrency cap.
- **Determinism:** pinned Typst version, bundled fonts, `--ignore-system-fonts`, and
  `SOURCE_DATE_EPOCH` for reproducible files.

---

## 8. The roadmap

The scores are a subjective "premium feel" rating against section 2. Each phase is shippable
alone.

### Phase 0: Fix what's broken (10 → 35)
- [x] Bundle the fonts; `--font-path` and `--ignore-system-fonts`; the font registry; a test asserting
      embedded fonts (B1).
- [x] Use `font_heading` for headings, and the mono font for code (B2).
- [x] Update the `chris-garlick-light` profile to the current brand (B6).
- [x] Inline formatting end to end: the markdown parser keeps Strong, Emphasis and Link; the Typst
      writer emits them (B5).
- [x] Upgrade Typst to 0.14 (B8).

*Done 20 Oct 2026 (9610232), deployed.* Unknown font names fall back by role rather than failing at profile save; revisit with the Phase 2 validation work.

*Acceptance:* a re-render of the Myers report shows Inter Tight headings and Inter text, bold works,
and `pdffonts` lists only bundled families.

### Phase 1: Typography and page system (35 → 55)
- [x] The token dictionary (section 3.1), emitted once and used everywhere.
- [x] The type scale, tracking, tabular figures, hyphenation, smart quotes.
- [x] The grid and margins; the running head with the section name ~~and logo mark~~; the new footer.
- [x] H2 no longer forces page breaks; sticky headings; widow and orphan control.
- [x] PDF bookmarks and metadata.

*Done 20 Oct 2026 (local; not yet deployed).* How it was built, and where it differs from the plan:
- **Theme file.** All style rules moved to `src/renderers/theme.typ` (the start of Phase 2's
  theme). The renderer emits `t` (tokens) and `meta` (document facts) above it. Old variable
  names stay as aliases for hand-written fletcher code.
- **Grid.** Simpler than a 12-column grid: the left margin holds a margin column a quarter of
  the text width wide. Reading text sits to its right (about 75 characters a line); tables that
  don't fit the text column, column layouts, the running head and rules extend across it. Typst
  measures each table to decide, so small tables stay in the text column.
- **Sections.** The shallowest heading level in the body opens a numbered section (01, 02… in
  the margin column, heavy rule above). Heading sizes are relative to that level, so H2-section
  documents (scout) and H1-section documents look the same. Section numbers come from the
  renderer, not a Typst counter.
- **Page breaks.** A section starts a new page only when less than 30% of the text area is left.
  This uses a fixed-height test block, not page positions: the position-based version needed a
  layout pass per section and gave up after 5, which scrambled the numbering (a test now guards
  this). JSON `pages` boundaries are soft section breaks (new `SectionBreak` node); `page_break`
  and `{{pagebreak}}` still force one.
- **Tables.** Column widths from content (short columns fit, long ones share by the square root of
  their average length); hairline rows, a heavy rule under the header, no vertical lines; no
  hyphenation in cells; tabular figures.
- **Boxes.** Styled sections and columns never split across pages; headings inside them are card
  titles in the box's text colour, left out of the bookmarks. This fixed the invisible title on
  dark callouts.
- **Cover.** The title shrinks to fit its longest word (the Myers domain no longer runs off the
  page).
- **Defaults.** The default profile uses the new scale (10.5 / 30 / 20 / 14, line height 1.55,
  22mm margins). `chris-garlick-light` moves to it, with paper `#FAF8F3` (decided 20 Oct: warm
  paper rather than white).
- **Moved to Phase 2:** the logo mark in the running head (it needs logo fetching, B3).
- **Tooling:** `cargo test preview -- --ignored` renders a local file with a profile override
  file, writing the PDF and its `.typ` source.

*Result:* the Myers report went from 14 pages to 10. Sections open in place when there is room.
The remaining white space is at section ends where the next section needs more room than is
left, or before blocks that stay whole.

*Acceptance:* no half-empty pages in the Myers report; headings never stranded; consistent rhythm.

### Phase 2: Core components (55 → 75)
- [ ] `theme.typ` with the cover (editorial and band), the section opener, the callout, the severity
      badge, the findings table, the step plan, the score row (gauges), metrics, facts.
- [ ] Logos rendered on the cover and in the running head (B3). The running head has room for
      the mark on the left (moved here from Phase 1).
- [ ] Schema v2 for these blocks, the JSON Schema and validation.
- [ ] `examples/gallery.json` and visual regression snapshots.

*Acceptance:* the gallery renders pixel-stable; a hand-written v2 Myers report looks like a design
studio made it.

### Phase 3: Data visuals and structure (75 → 88)
- [ ] The score matrix (heat table), bar comparison, severity summary, priority matrix.
- [ ] The contents page and back page; finding cards for headline issues.
- [ ] Images and figures with safe fetching (B4).

*Acceptance:* the Myers summary fits on one "at a glance" page: score gauges, a severity summary,
the top three findings as cards, and a priority matrix.

### Phase 4: Wire it into scout (88 → 94)
- [ ] `typeset-format.md` rewritten around v2 components; the scout validator follows the JSON
      Schema.
- [ ] scout passes structured data (scores per page, measurements, counts) straight into
      components (`score_matrix`, `bars`, `metrics`), so Claude places components rather than
      re-typing numbers. Fewer chances to misstate a figure.
- [ ] `/api/proof` thumbnails used by `scout review` to check layout before saving.

*Acceptance:* a fresh `scout review` produces the premium layout with no hand edits.

### Phase 5: Polish and proof (94 → 100)
- [ ] Optical details: hung punctuation in quotes, optical alignment of numerals, refined
      kerning in display sizes, consistent icon set (a small SVG set in the theme).
- [ ] Tagged PDF/UA, alt text enforcement, contrast tests for every profile.
- [ ] Print checks: a CMYK-safe palette option and a bleed variant of the cover for professional
      printing.
- [ ] Final design review against section 2 with three real reports (website review, proposal,
      strategy doc); fix everything that doesn't hold up beside a consultancy deck.

*Acceptance:* side by side with a premium consultancy report, it holds its own. No one can tell
it was generated.

---

## 9. Order of work and size

| Phase | Size | Notes |
|---|---|---|
| 0 | 1 to 2 days | Mostly config and a parser fix. The biggest visible jump. |
| 1 | 2 to 3 days | Template work in Typst; easy to iterate with `typst watch`. |
| 2 | 4 to 6 days | The bulk: components, schema v2, gallery, snapshots. |
| 3 | 3 to 4 days | Drawing in Typst (arcs, bars, matrix) and image fetching. |
| 4 | 1 to 2 days | In the scout repo: format guide, validator, structured data. |
| 5 | 2 to 3 days | Detail and review. |

**Do Phase 0 this week.** It fixes the font fallback that every current PDF suffers from,
before any new design work.

---

## 10. Risks and decisions

- **Backward compatibility:** schema v1 documents (Kritano exports, older scout reviews) must render
  unchanged. A snapshot of each existing example guards this.
- **Typst upgrades:** pin the version; upgrade deliberately with snapshot review.
- **Design ownership:** the theme is plain Typst in the repo, so design changes are small diffs that
  can be previewed with `typst watch` and reviewed as images.
- **Decided (20 Oct 2026):** the paper tint is `#FAF8F3`, matching the website.
- **Decision needed:** the cover style default. Recommend "editorial" (light, logo, strong
  typography) for reviews and proposals; "band" for longer strategy documents.
