// Typeset components (schema v2). Each function draws one JSON block type;
// the renderer passes the block's fields as arguments, with text already
// converted to content. Reads `t` and `meta` like the theme.

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

#let status-color(status) = if status == none { t.ink-muted } else {
  (good: t.good, fair: t.fair, poor: t.poor).at(status, default: t.ink-muted)
}

/// Status from a 0 to 100 score, with Lighthouse's thresholds.
#let score-status(percent) = if percent >= 90 { "good" } else if percent >= 50 { "fair" } else { "poor" }

/// A colour mixed into the paper: the tinted surface behind toned content.
#let tint(c, amount: 88%) = color.mix((c, 100% - amount), (t.paper, amount))

#let eyebrow-text(body, fill: t.ink-muted) = text(
  size: t.eyebrow,
  weight: 600,
  tracking: 0.1em,
  fill: fill,
  upper(body),
)

#let chip(body, fill: t.surface, color: t.ink) = box(
  fill: fill,
  inset: (x: 5pt, y: 2pt),
  outset: (y: 1pt),
  radius: t.r-sm,
  text(size: t.eyebrow, fill: color, body),
)

#let severity-colors = (
  critical: t.poor,
  serious: rgb("#C2410C"),
  moderate: t.fair,
  minor: t.ink-muted,
)

/// A pill with a dot and the severity in tracked capitals. Critical is
/// filled; the rest are tinted.
#let severity-badge(level) = {
  let c = severity-colors.at(level, default: t.ink-muted)
  let filled = level == "critical"
  let ink = if filled { t.paper } else { c.darken(10%) }
  box(
    fill: if filled { c } else { tint(c, amount: 85%) },
    inset: (x: 6pt, y: 2.5pt),
    outset: (y: 1pt),
    radius: 99pt,
  )[#box(baseline: -1.5pt, circle(radius: 2.2pt, fill: if filled { t.paper } else { c }))#h(4pt)#text(
      size: t.eyebrow * 0.92,
      weight: 600,
      tracking: 0.08em,
      fill: ink,
      hyphenate: false,
      upper(level),
    )]
}

// ---------------------------------------------------------------------------
// Covers
// ---------------------------------------------------------------------------

#let cover-meta(fields, color: t.ink, muted: t.ink-muted, rule: t.rule) = {
  let filled = fields.filter(f => f.at(1) != none)
  if filled.len() == 0 { return }
  block(width: 100%, inset: (top: t.s4), stroke: (top: t.hairline + rule), grid(
    columns: (1fr,) * calc.max(filled.len(), 2),
    column-gutter: t.s5,
    ..filled.map(f => {
      block(below: t.s2, eyebrow-text(f.at(0), fill: muted))
      block(text(size: t.body, fill: color, f.at(1)))
    }),
  ))
}

/// The cover. `style` is `editorial` (paper, brand mark, strong type),
/// `band` (the same under an accent band) or `dark` (a dark full page).
#let cover(
  style: "editorial",
  title: none,
  title-size: 40pt,
  subtitle: none,
  eyebrow: none,
  recipient: none,
  author: none,
  date: none,
  reference: none,
) = {
  let fields = (("Prepared for", recipient), ("Prepared by", author), ("Date", date), ("Reference", reference))
  let pad-box = (left: t.margin-left, right: t.margin-right, top: t.margin-top, bottom: t.margin-bottom)

  if style == "dark" {
    page(margin: 0mm, header: none, footer: none, fill: t.cover-bg)[
      #set text(fill: t.cover-text, hyphenate: false)
      #pad(..pad-box)[
        #brand-mark(height: 12pt, dark: true, color: t.cover-text)
        #v(1fr)
        #if eyebrow != none { eyebrow-text(eyebrow, fill: t.cover-text.transparentize(35%)); v(t.s4) }
        #block(width: 100%, text(font: t.font-heading, size: title-size, weight: 600, tracking: -0.02em, title))
        #v(t.s5)
        #line(length: 24mm, stroke: 3pt + t.accent.lighten(35%))
        #if subtitle != none { v(t.s5); block(width: 75%, text(size: t.h3, fill: t.cover-text.transparentize(25%), subtitle)) }
        #v(1.3fr)
        #cover-meta(fields, color: t.cover-text, muted: t.cover-text.transparentize(45%), rule: t.cover-text.transparentize(75%))
      ]
    ]
  } else if style == "band" {
    page(margin: 0mm, header: none, footer: none, fill: t.paper)[
      #set text(hyphenate: false)
      #block(width: 100%, height: 46%, fill: t.accent, pad(..pad-box)[
        #set text(fill: t.paper)
        #brand-mark(height: 12pt, dark: true, color: t.paper)
        #v(1fr)
        #if eyebrow != none { eyebrow-text(eyebrow, fill: t.paper.transparentize(25%)); v(t.s4) }
        #block(width: 100%, text(font: t.font-heading, size: title-size, weight: 600, tracking: -0.02em, title))
      ])
      #pad(left: t.margin-left, right: t.margin-right, top: t.s7, bottom: t.margin-bottom)[
        #if subtitle != none { block(width: 80%, text(size: t.h3, fill: t.ink-muted, subtitle)) }
        #v(1fr)
        #cover-meta(fields)
      ]
    ]
  } else {
    page(margin: 0mm, header: none, footer: none, fill: t.paper)[
      #set text(hyphenate: false)
      #pad(..pad-box)[
        #brand-mark(height: 12pt)
        #v(t.s3)
        #line(length: 100%, stroke: t.heavy + t.ink)
        #v(1fr)
        #if eyebrow != none { eyebrow-text(eyebrow, fill: t.accent); v(t.s4) }
        #block(width: 100%, text(font: t.font-heading, size: title-size, weight: 600, tracking: -0.02em, fill: t.heading, title))
        #v(t.s5)
        #line(length: 24mm, stroke: 3pt + t.accent)
        #if subtitle != none { v(t.s5); block(width: 80%, text(size: t.h3, fill: t.ink-muted, subtitle)) }
        #v(1.3fr)
        #cover-meta(fields)
      ]
    ]
  }
}

// ---------------------------------------------------------------------------
// Section opener
// ---------------------------------------------------------------------------

#let stat-block(stat) = {
  let c = status-color(stat.status)
  align(right, {
    block(below: t.s2, text(font: t.font-heading, size: t.h1, weight: 600, tracking: -0.02em, fill: if stat.status == none { t.heading } else { c }, number-width: "tabular", stat.value))
    if stat.label != none { block(eyebrow-text(stat.label)) }
  })
}

/// A section opening with an eyebrow, the title (a real heading, so it is
/// numbered, bookmarked and named in the running head), an intro and an
/// optional headline figure.
#let opener(number: none, level: 1, title: none, eyebrow: none, intro: none, long-intro: false, stat: none) = {
  section-break()
  block(sticky: true, above: 0pt, below: t.s6, width: 100%, wide[
    #line(length: 100%, stroke: t.heavy + t.ink)
    #v(t.s5)
    #grid(
      columns: (t.side, 1fr, auto),
      column-gutter: (0pt, t.s5),
      text(font: t.font-heading, size: t.h3, weight: 600, fill: t.accent, number-width: "tabular", if number != none { number }),
      {
        if eyebrow != none { block(below: t.s3, eyebrow-text(eyebrow, fill: t.accent)) }
        show heading: it => block(text(font: t.font-heading, size: t.h1, weight: 600, tracking: -0.015em, fill: t.heading, hyphenate: false, it.body))
        set par(leading: 0.3em)
        heading(level: level, supplement: if number != none { [#number] }, title)
        if intro != none {
          set par(leading: t.leading)
          // A long intro reads better at body size.
          block(above: t.s4, text(size: if long-intro { t.body } else { t.body * 1.15 }, fill: t.ink, intro))
        }
      },
      if stat != none { stat-block(stat) },
    )
  ])
}

// ---------------------------------------------------------------------------
// Callout
// ---------------------------------------------------------------------------

#let tone-color(tone) = (
  good: t.good,
  warn: t.fair,
  critical: t.poor,
  brand: t.accent,
).at(tone, default: t.ink-muted)

#let tone-icon(tone, c) = box(baseline: 20%, circle(radius: 6.5pt, fill: c, stroke: none, align(center + horizon, {
  if tone == "good" {
    curve(stroke: (paint: t.paper, thickness: 1.4pt, cap: "round", join: "round"), curve.move((0pt, 3.2pt)), curve.line((2.3pt, 5.4pt)), curve.line((6.4pt, 0.6pt)))
  } else {
    text(size: 8pt, weight: 700, fill: t.paper, if tone == "info" { "i" } else if tone == "brand" { "→" } else { "!" })
  }
})))

/// A tinted note with a rule in its tone's colour. Never a dark fill.
#let callout(tone: "info", title: none, body) = {
  let c = tone-color(tone)
  block(
    width: 100%,
    breakable: false,
    above: t.s6,
    below: t.s6,
    fill: tint(c, amount: 90%),
    stroke: (left: 3pt + c),
    radius: (right: t.r-md),
    inset: (left: t.s5, right: t.s5, y: t.s4),
  )[
    #if title != none {
      block(below: t.s3, sticky: true, grid(columns: (auto, 1fr), column-gutter: t.s3, align: horizon,
        tone-icon(tone, c),
        text(font: t.font-heading, size: t.h3 * 0.93, weight: 600, fill: t.ink, title)))
    }
    #body
  ]
}

// ---------------------------------------------------------------------------
// Findings table
// ---------------------------------------------------------------------------

/// Findings as a table: severity badge, the issue (with how often it was
/// found), where, and what to do. Zebra rows, hairline separators.
#let findings(items) = wide(table(
  columns: (auto, 1.5fr, 0.9fr, 1.6fr),
  inset: (x: t.s3, y: 7pt),
  fill: (x, y) => if y == 0 { t.surface } else if calc.even(y) { t.surface.transparentize(55%) },
  align: top,
  table.header([Severity], [Issue], [Where], [What to do]),
  ..items.map(f => (
    severity-badge(f.severity),
    {
      block(below: t.s2, text(weight: 500, fill: t.ink, f.title))
      if f.count != none { block(text(fill: t.ink-muted, size: t.eyebrow)[Found #f.count times]) }
    },
    // Spaces between chips are where lines can break.
    f.pages.map(p => chip(p)).join(" "),
    text(fill: t.ink, if f.fix != none { f.fix }),
  )).flatten(),
))

// ---------------------------------------------------------------------------
// Step plan
// ---------------------------------------------------------------------------

/// Numbered steps in groups. The numbers hang in the margin column and run on
/// across groups; no leading zero, so they don't read as section numbers.
#let steps(groups) = {
  let n = 0
  for g in groups {
    if g.title != none {
      block(above: t.s6, below: t.s3, sticky: true, width: 100%, inset: (bottom: t.s2), stroke: (bottom: t.hairline + t.rule),
        text(font: t.font-heading, size: t.h3, weight: 600, fill: t.ink, g.title))
    }
    for s in g.items {
      n += 1
      block(above: t.s4, below: t.s4, breakable: false, wide(grid(
        columns: (t.side, 1fr, auto),
        column-gutter: (0pt, t.s4),
        move(dy: -2.5pt, text(font: t.font-heading, size: t.h3, weight: 600, fill: t.accent, number-width: "tabular", str(n))),
        {
          block(below: t.s2, text(weight: 600, fill: t.ink, s.title))
          if s.detail != none { block(text(fill: t.ink-muted, s.detail)) }
        },
        if s.effort != none { chip(s.effort) },
      )))
    }
  }
}

// ---------------------------------------------------------------------------
// Scores
// ---------------------------------------------------------------------------

/// A 270° gauge: a track, the value's arc in its status colour, the figure in
/// the middle and an optional benchmark tick.
#let gauge(value, max: 100, benchmark: none, size: 76pt) = {
  let percent = calc.clamp(value / max * 100, 0, 100)
  let c = status-color(score-status(percent))
  let r = size / 2 - 5pt
  let mid = (size / 2, size / 2)
  let point(a, radius: r) = (mid.at(0) + radius * calc.cos(a), mid.at(1) - radius * calc.sin(a))
  let arc(from, to, paint, steps: 60) = {
    let pts = range(steps + 1).map(i => point(from + (to - from) * i / steps))
    curve(stroke: (paint: paint, thickness: 5pt, cap: "round"), curve.move(pts.first()), ..pts.slice(1).map(p => curve.line(p)))
  }
  box(width: size, height: size * 0.86, {
    place(arc(225deg, -45deg, t.rule))
    if percent > 0 { place(arc(225deg, 225deg - 270deg * percent / 100, c)) }
    if benchmark != none {
      let a = 225deg - 270deg * calc.clamp(benchmark / max, 0, 1)
      place(curve(stroke: 1.2pt + t.ink, curve.move(point(a, radius: r - 5pt)), curve.line(point(a, radius: r + 5pt))))
    }
    place(center + horizon, dy: -2pt, text(font: t.font-heading, size: size * 0.3, weight: 600, fill: t.ink, number-width: "tabular", str(calc.round(value))))
    place(bottom + center, dy: -2pt, text(size: t.eyebrow, fill: t.ink-muted, number-width: "tabular")[of #str(calc.round(max))])
  })
}

/// Two to five gauges across the grid, each on a card, with a caption.
#let scores(items, caption: none) = {
  block(above: t.s6, below: t.s6, breakable: false, wide({
    grid(
      columns: (1fr,) * items.len(),
      column-gutter: t.s3,
      ..items.map(s => block(width: 100%, fill: t.surface, radius: t.r-md, inset: (x: t.s3, top: t.s4, bottom: t.s4), align(center)[
        #gauge(s.value, max: s.max, benchmark: s.benchmark)
        #v(t.s2, weak: true)
        #text(font: t.font-heading, size: t.body, weight: 600, fill: t.ink, s.label)
      ])),
    )
    if caption != none { block(above: t.s3, text(size: t.small, fill: t.ink-muted, caption)) }
  }))
}

// ---------------------------------------------------------------------------
// Metrics and facts
// ---------------------------------------------------------------------------

/// Headline figures: a status rule, the value large, the label, the target.
#let metrics(items) = block(above: t.s6, below: t.s6, breakable: false, wide(grid(
  columns: (1fr,) * calc.min(items.len(), 4),
  column-gutter: t.s5,
  row-gutter: t.s5,
  ..items.map(m => block(width: 100%, inset: (top: t.s4), stroke: (top: 2.5pt + status-color(m.status)), {
    block(below: t.s3, text(font: t.font-heading, size: t.h2 * 1.25, weight: 600, tracking: -0.01em, fill: t.ink, number-width: "tabular", m.value))
    block(below: t.s2, text(size: t.small, weight: 600, fill: t.ink, m.label))
    if m.target != none { block(text(size: t.eyebrow, fill: t.ink-muted)[Target: #m.target]) }
  })),
)))

/// A definition list: labels in the margin column, values in the text column.
#let facts(items) = block(above: t.s5, below: t.s5, wide(grid(
  columns: (t.side, 1fr),
  row-gutter: 0pt,
  ..items.map(f => (
    block(width: 100%, inset: (top: t.s3 + 1.5pt, bottom: t.s3), stroke: (top: t.hairline + t.rule), eyebrow-text(f.label)),
    block(width: 100%, inset: (y: t.s3), stroke: (top: t.hairline + t.rule), f.value),
  )).flatten(),
)))
