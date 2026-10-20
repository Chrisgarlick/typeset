// Typeset theme.
//
// Everything here reads from two dictionaries the renderer emits above this
// file: `t` (design tokens: colours, sizes, spacing, the grid) and `meta`
// (facts about this document: title, cover, which heading level opens a
// section). Nothing below hard-codes a colour or a size.

// Names from before the token dictionary. Hand-written Typst in fletcher
// code blocks may still use them.
#let primary = t.heading
#let secondary = t.ink-muted
#let accent = t.accent
#let text-color = t.ink
#let bg-color = t.paper
#let table-header-bg = t.surface
#let table-border-color = t.rule
#let callout-bg = t.accent-soft
#let cover-bg-color = t.cover-bg
#let cover-text-color = t.cover-text
#let font-body = t.font-body
#let font-heading = t.font-heading
#let font-mono = t.font-mono
#let mono-stack = (t.font-mono,)

#set document(title: meta.title, author: meta.author, description: meta.description)

// ---------------------------------------------------------------------------
// The grid
//
// The page's left margin includes a margin column (`t.side`). Reading text
// sits in the text column to its right; wide things (tables, column layouts,
// the running head) reach back across the margin column with `wide`.
// ---------------------------------------------------------------------------

#let wide(body) = pad(left: -t.side, body)

/// The current section's name for the running head: the section that opens
/// on this page, or else the last one before it.
#let section-name() = {
  let sel = heading.where(level: meta.section-level, outlined: true)
  let n = here().page()
  let on-page = query(sel).filter(h => h.location().page() == n)
  if on-page.len() > 0 {
    on-page.first().body
  } else {
    let before = query(sel.before(here()))
    if before.len() > 0 { before.last().body }
  }
}

#set page(
  paper: meta.paper,
  fill: t.paper,
  margin: (
    top: t.margin-top,
    bottom: t.margin-bottom,
    left: t.margin-left + t.side,
    right: t.margin-right,
  ),
  header-ascent: 35%,
  footer-descent: 35%,
  header: if meta.header {
    context wide(block(width: 100%, inset: (bottom: t.s3), stroke: (bottom: t.hairline + t.rule))[
      #set text(font: t.font-body, size: t.eyebrow, fill: t.ink-muted, hyphenate: false)
      #grid(columns: (1fr, auto), column-gutter: t.s5, meta.running-title, section-name())
    ])
  },
  footer: if meta.footer {
    context wide(block(width: 100%, inset: (top: t.s3), stroke: (top: t.hairline + t.rule))[
      #set text(font: t.font-body, size: t.eyebrow, fill: t.ink-muted, number-width: "tabular", hyphenate: false)
      #let n = counter(page).get().first()
      #let total = counter(page).final().first()
      #let (n, total) = if meta.has-cover { (n - 1, total - 1) } else { (n, total) }
      #grid(columns: (1fr, auto), column-gutter: t.s5, meta.footer-left, [Page #n of #total])
    ])
  },
)

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

#set text(
  font: t.font-body,
  size: t.body,
  fill: t.ink,
  lang: "en",
  region: "gb",
  hyphenate: true,
  costs: (widow: 100%, orphan: 100%, hyphenation: 200%),
)
#set par(leading: t.leading, spacing: t.par-spacing, justify: false, linebreaks: "optimized")
#set block(spacing: t.par-spacing)
#set list(marker: text(fill: t.accent)[•], indent: 1pt, body-indent: t.s3, spacing: t.s3)
#set enum(
  numbering: n => text(fill: t.accent, weight: 600, number-width: "tabular")[#n.],
  indent: 0pt,
  body-indent: t.s3,
  spacing: t.s3,
)
#set strong(delta: 200)
#show strong: set text(weight: 600)
#show link: set text(fill: t.accent)
#show link: underline.with(stroke: 0.5pt + t.accent.transparentize(50%), offset: 2pt)

// ---------------------------------------------------------------------------
// Sections and headings
//
// The top heading level in the document opens a section. A section starts a
// new page only when the current one is mostly used; otherwise it opens in
// place below a heavy rule, so short sections don't leave half-empty pages.
// Lower levels are sized relative to it, so a document whose sections are H2s
// looks the same as one whose sections are H1s.
// ---------------------------------------------------------------------------

/// Open a section: move to a new page unless `t.section-need` of space is
/// left. The invisible block takes that space only to test the fit, then the
/// negative spacing gives it back; no page positions are read, so layout
/// settles in one pass however many sections there are.
#let section-break() = {
  v(t.s8, weak: true)
  block(width: 100%, height: t.section-need, breakable: false, above: 0pt, below: 0pt)
  v(-t.section-need)
}

/// Section numbers come from the renderer as the heading's supplement.
#let section-number(it) = {
  let s = it.supplement
  if type(s) == content and s.has("text") and s.text.match(regex("^\d+$")) != none { s.text }
}

#show heading: set par(leading: 0.3em)
#show heading: it => {
  let rel = calc.max(1, it.level - meta.section-level + 1)
  if rel == 1 {
    section-break()
    block(sticky: true, above: 0pt, below: t.s6, width: 100%, wide[
      #line(length: 100%, stroke: t.heavy + t.ink)
      #v(t.s5)
      #set par(hanging-indent: t.side, leading: 0.25em)
      #box(width: t.side, text(
        font: t.font-heading,
        size: t.h3,
        weight: 600,
        fill: t.accent,
        number-width: "tabular",
        section-number(it),
      ))#text(
        font: t.font-heading,
        size: t.h1,
        weight: 600,
        tracking: -0.015em,
        fill: t.heading,
        hyphenate: false,
        it.body,
      )
    ])
  } else {
    let (size, above, below, tracking, fill) = if rel == 2 {
      (t.h2, t.s7, t.s3, -0.01em, t.heading)
    } else if rel == 3 {
      (t.h3, t.s6, t.s2, -0.005em, t.ink)
    } else {
      (t.body * 1.1, t.s5, t.s2, 0em, t.ink)
    }
    block(sticky: true, above: above, below: below, text(
      font: t.font-heading,
      size: size,
      weight: 600,
      tracking: tracking,
      fill: fill,
      hyphenate: false,
      it.body,
    ))
  }
}

/// Headings inside a styled box (a card or callout): small, never bookmarked,
/// and in whatever text colour the box set.
#let boxed-heading(it) = block(sticky: true, above: t.s4, below: t.s3, text(
  font: t.font-heading,
  size: t.h3,
  weight: 600,
  hyphenate: false,
  it.body,
))

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

#set table(
  stroke: (x, y) => (bottom: if y == 0 { 0.8pt + t.ink } else { t.hairline + t.rule }),
  inset: (x: t.s3, y: 6pt),
  fill: (x, y) => if y == 0 { t.surface },
)
#show table: set text(size: t.small, number-width: "tabular", hyphenate: false)
#show table: set par(leading: 0.5em)
#show table: set block(breakable: true)
#show table.cell.where(y: 0): set text(weight: 600, size: t.eyebrow, tracking: 0.02em)

/// A table that fits the text column at its natural width stays there; one
/// that doesn't spans the full grid with `flex` column widths.
#let fit-table(flex, ..cells) = context {
  let text-width = page.width - t.margin-left - t.side - t.margin-right
  let natural = measure(table(columns: flex.len() * (auto,), ..cells.pos())).width
  if natural <= text-width {
    table(columns: flex.len() * (auto,), ..cells.pos())
  } else {
    wide(table(columns: flex, ..cells.pos()))
  }
}

// ---------------------------------------------------------------------------
// Quotes, code, rules
// ---------------------------------------------------------------------------

#show quote.where(block: true): it => block(
  width: 100%,
  above: t.s6,
  below: t.s6,
  inset: (left: t.s5, y: t.s2),
  stroke: (left: t.heavy + t.accent),
)[
  #set text(size: t.body * 1.08, fill: t.ink)
  #it.body
]

#show raw.where(block: true): it => block(
  breakable: false,
  width: 100%,
  fill: rgb("#1F1F23"),
  inset: (x: t.s4, y: t.s4),
  radius: t.r-md,
  above: t.s6,
  below: t.s6,
)[
  #set text(fill: rgb("#E5E7EB"), font: mono-stack, size: t.small)
  #set par(leading: 0.55em, justify: false)
  #it
]

#show raw.where(block: false): it => box(
  fill: t.surface,
  inset: (x: 3pt, y: 0pt),
  outset: (y: 3pt),
  radius: t.r-sm,
)[#text(font: mono-stack, size: t.small)[#it]]

#let hairline() = wide(line(length: 100%, stroke: t.hairline + t.rule))
