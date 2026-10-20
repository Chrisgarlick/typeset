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
#let tint(c, amount: 88%) = blend(c, t.paper, amount)

#let eyebrow-text(body, fill: t.ink-muted) = text(
  size: t.eyebrow,
  weight: 600,
  tracking: 0.1em,
  fill: fill,
  upper(body),
)

/// A display figure that shrinks to fit its column on one line, down to
/// `min` (a long range such as "5,948 to 6,800 ms" must not wrap).
#let fit-figure(body, size: t.h2, min: t.body * 1.1, fill: t.ink, weight: 600) = layout(region => {
  // Proportional figures: a lone display number shouldn't have tabular gaps.
  let set-in(s) = text(font: t.font-heading, size: s, weight: weight, tracking: -0.01em, fill: fill, hyphenate: false, body)
  let natural = measure(set-in(size)).width
  let fitted = if natural > region.width and natural > 0pt { calc.max(min / 1pt, size / 1pt * region.width / natural) * 1pt } else { size }
  set-in(fitted)
})

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

/// The cover title as the document's one level 1 heading (tagged PDF needs
/// one), drawn as plain display text and left out of the bookmarks.
#let title-heading(title, size: 40pt, fill: auto) = {
  // Styled inside the show rule: Typst's built-in heading style would
  // otherwise enlarge it.
  show heading: it => text(
    font: t.font-heading,
    size: size,
    weight: 600,
    tracking: -0.02em,
    ..if fill != auto { (fill: fill) },
    it.body,
  )
  heading(level: 1, outlined: false, bookmarked: false, title)
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
        #if eyebrow != none { eyebrow-text(eyebrow, fill: blend(t.cover-text, t.cover-bg, 35%)); v(t.s4) }
        #block(width: 100%, title-heading(title, size: title-size))
        #v(t.s5)
        #line(length: 24mm, stroke: 3pt + t.accent.lighten(35%))
        #if subtitle != none { v(t.s5); block(width: 75%, text(size: t.h3, fill: blend(t.cover-text, t.cover-bg, 25%), subtitle)) }
        #v(1.3fr)
        #cover-meta(fields, color: t.cover-text, muted: blend(t.cover-text, t.cover-bg, 45%), rule: blend(t.cover-text, t.cover-bg, 75%))
      ]
    ]
  } else if style == "band" {
    page(margin: 0mm, header: none, footer: none, fill: t.paper)[
      #set text(hyphenate: false)
      #block(width: 100%, height: 46%, fill: t.accent, pad(..pad-box)[
        #set text(fill: t.paper)
        #brand-mark(height: 12pt, dark: true, color: t.paper)
        #v(1fr)
        #if eyebrow != none { eyebrow-text(eyebrow, fill: blend(t.paper, t.accent, 25%)); v(t.s4) }
        #block(width: 100%, title-heading(title, size: title-size))
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
        #block(width: 100%, title-heading(title, size: title-size, fill: t.heading))
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
    block(below: t.s2, text(font: t.font-heading, size: t.h1, weight: 600, tracking: -0.02em, fill: if stat.status == none { t.heading } else { c }, stat.value))
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

/// The icon set: small marks drawn in paper colour on a filled circle,
/// one per tone, all with the same stroke, so they read as a family.
#let tone-icon(tone, c) = {
  let k = (paint: t.paper, thickness: 1.4pt, cap: "round", join: "round")
  let mark = if tone == "good" {
    // A tick.
    curve(stroke: k, curve.move((-3.1pt, 0.2pt)), curve.line((-0.9pt, 2.4pt)), curve.line((3.2pt, -2.2pt)))
  } else if tone == "info" {
    // A lower-case i: a dot over a stem.
    place(center + horizon, dy: -2.6pt, circle(radius: 0.85pt, fill: t.paper))
    curve(stroke: k, curve.move((0pt, -0.6pt)), curve.line((0pt, 3.2pt)))
  } else if tone == "brand" {
    // An arrow pointing on.
    curve(stroke: k, curve.move((-3pt, 0pt)), curve.line((3pt, 0pt)), curve.move((0.6pt, -2.4pt)), curve.line((3pt, 0pt)), curve.line((0.6pt, 2.4pt)))
  } else if tone == "critical" {
    // A cross.
    curve(stroke: k, curve.move((-2.4pt, -2.4pt)), curve.line((2.4pt, 2.4pt)), curve.move((2.4pt, -2.4pt)), curve.line((-2.4pt, 2.4pt)))
  } else {
    // An exclamation mark: a stem over a dot.
    place(center + horizon, dy: 2.8pt, circle(radius: 0.85pt, fill: t.paper))
    curve(stroke: k, curve.move((0pt, -3.2pt)), curve.line((0pt, 0.8pt)))
  }
  box(baseline: 20%, width: 13pt, height: 13pt, {
    place(circle(radius: 6.5pt, fill: c))
    place(center + horizon, box(width: 0pt, height: 0pt, place(dx: 0pt, dy: 0pt, mark)))
  })
}

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
  fill: (x, y) => if y == 0 { t.surface } else if calc.even(y) { blend(t.surface, t.paper, 55%) },
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
#let gauge(value, max: 100, benchmark: none, size: 68pt) = {
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
    place(center + horizon, dy: -2pt, text(font: t.font-heading, size: size * 0.3, weight: 600, fill: t.ink, str(calc.round(value))))
    place(bottom + center, dy: -2pt, text(size: t.eyebrow, fill: t.ink-muted, number-width: "tabular")[of #str(calc.round(max))])
  })
}

/// Two to five gauges across the grid, each on a card, with a caption.
#let scores(items, caption: none) = {
  block(above: t.s5, below: t.s5, breakable: false, wide({
    grid(
      columns: (1fr,) * items.len(),
      column-gutter: t.s3,
      ..items.map(s => block(width: 100%, fill: t.surface, radius: t.r-md, inset: (x: t.s3, top: t.s3, bottom: t.s3), align(center)[
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
    block(below: t.s3, fit-figure(m.value, size: t.h2 * 1.25))
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

// ---------------------------------------------------------------------------
// Data visuals
// ---------------------------------------------------------------------------

/// A score as a tinted pill, coloured by status; `none` shows a dash.
#let score-pill(value, max: 100) = if value == none {
  text(fill: t.ink-muted)[–]
} else {
  let c = status-color(score-status(value / max * 100))
  box(fill: tint(c, amount: 82%), inset: (x: 8pt, y: 3pt), outset: (y: 1pt), radius: 99pt, text(
    weight: 600,
    fill: c.darken(20%),
    number-width: "tabular",
    str(calc.round(value)),
  ))
}

/// Scores by page and category as a heat table.
#let score-matrix(columns, rows, max: 100, caption: none) = block(above: t.s6, below: t.s6, wide({
  table(
    columns: (1.6fr,) + (1fr,) * columns.len(),
    align: (x, y) => if x == 0 { left + horizon } else { center + horizon },
    inset: (x: t.s3, y: 7pt),
    table.header([Page], ..columns),
    ..rows.map(r => (text(size: t.small, r.label), ..r.values.map(v => score-pill(v, max: max)))).flatten(),
  )
  if caption != none { block(above: t.s3, text(size: t.small, fill: t.ink-muted, caption)) }
}))

/// Horizontal bars, with a dashed target line. Bars past the target (in the
/// wrong direction) are poor; within it, good; with no target, accent.
#let bars(items, unit: none, target: none, better: "lower", caption: none) = block(above: t.s6, below: t.s6, breakable: false, wide({
  let peak = calc.max(..items.map(i => i.value), if target == none { 0 } else { target })
  let scale = 82%
  let row-height = 20pt
  let fmt(v) = str(v) + if unit != none { unit } else { "" }
  let color(v) = if target == none { t.accent } else if (better == "lower" and v <= target) or (better == "higher" and v >= target) { t.good } else { t.poor }
  grid(
    columns: (t.side, 1fr),
    stack(..items.map(i => block(height: row-height, width: 100%, align(horizon, text(size: t.small, fill: t.ink, i.label))))),
    layout(region => block(width: 100%, {
      let target-x = if target != none { region.width * scale * (target / peak) }
      if target != none {
        place(top + left, dx: target-x, dy: -3pt, line(angle: 90deg, length: row-height * items.len() + 6pt, stroke: (paint: t.ink, thickness: 0.8pt, dash: "dashed")))
        place(top + left, dx: target-x + 3pt, dy: -12pt, text(size: t.eyebrow, fill: t.ink-muted)[Target #fmt(target)])
      }
      stack(..items.map(i => block(height: row-height, width: 100%, align(horizon, {
        let bar-end = region.width * scale * (i.value / peak)
        box(width: bar-end, height: 9pt, fill: color(i.value), radius: (right: 2pt))
        // A label that would sit across the target line starts past it.
        let gap = if target != none and target-x > bar-end and target-x - bar-end < 36pt { target-x - bar-end + 4pt } else { t.s2 }
        h(gap)
        text(size: t.small, weight: 600, fill: t.ink, number-width: "tabular", fmt(i.value))
      }))))
    })),
  )
  if caption != none { block(above: t.s4, text(size: t.small, fill: t.ink-muted, caption)) }
}))

/// Issue counts by severity: a segmented bar over four counts.
#let severity-summary(counts, caption: none) = block(above: t.s5, below: t.s5, breakable: false, wide({
  let levels = ("critical", "serious", "moderate", "minor")
  let present = levels.filter(l => counts.at(l) > 0)
  if present.len() > 0 {
    grid(
      columns: present.map(l => counts.at(l) * 1fr),
      column-gutter: 2pt,
      ..present.map(l => rect(width: 100%, height: 8pt, fill: severity-colors.at(l), radius: 2pt)),
    )
  }
  v(t.s3)
  grid(
    columns: (1fr,) * 4,
    column-gutter: t.s4,
    ..levels.map(l => {
      block(below: t.s2, text(font: t.font-heading, size: t.h2, weight: 600, fill: if counts.at(l) > 0 { severity-colors.at(l) } else { t.ink-muted }, str(counts.at(l))))
      severity-badge(l)
    }),
  )
  if caption != none { block(above: t.s3, text(size: t.small, fill: t.ink-muted, caption)) }
}))

// ---------------------------------------------------------------------------
// Finding card and priority matrix
// ---------------------------------------------------------------------------

/// A headline issue: a severity rule, the badge and where, the title, then
/// why it matters and what to do side by side, with evidence on the right.
#let finding-card(title: none, severity: "moderate", why: none, fix: none, pages: (), evidence: none) = block(
  above: t.s4,
  below: t.s4,
  breakable: false,
  wide(block(
    width: 100%,
    fill: t.surface,
    radius: (right: t.r-md),
    stroke: (left: 4pt + severity-colors.at(severity, default: t.ink-muted)),
    inset: (left: t.s5, right: t.s5, y: t.s4),
  )[
    #grid(
      columns: (1fr, auto),
      column-gutter: t.s5,
      {
        block(below: t.s3, { severity-badge(severity); if pages.len() > 0 { h(t.s3); pages.map(p => chip(p, fill: t.paper)).join(" ") } })
        block(text(font: t.font-heading, size: t.h3, weight: 600, fill: t.ink, title))
      },
      if evidence != none {
        align(right, {
          block(below: t.s1, width: 38mm, align(right, fit-figure(evidence.value, size: t.h2, fill: severity-colors.at(severity, default: t.ink))))
          if evidence.label != none { block(text(size: t.eyebrow, fill: t.ink-muted, evidence.label)) }
        })
      },
    )
    #{
      let parts = ()
      if why != none { parts.push({ block(below: t.s2, eyebrow-text("Why it matters")); why }) }
      if fix != none { parts.push({ block(below: t.s2, eyebrow-text("What to do")); fix }) }
      // Side by side when there are both; otherwise one full-width column.
      if parts.len() > 0 { block(above: t.s3, grid(columns: (1fr,) * parts.len(), column-gutter: t.s6, ..parts)) }
    }
  ]),
)

/// Up to four headline findings side by side: a severity rule, the badge,
/// the evidence large, the title and the fix. Equal heights.
#let highlights(items) = block(above: t.s5, below: t.s5, breakable: false, wide(grid(
  columns: (1fr,) * items.len(),
  column-gutter: t.s3,
  fill: t.surface,
  inset: (x: t.s4, top: t.s4, bottom: t.s4),
  stroke: (x, y) => (top: 3pt + severity-colors.at(items.at(x).severity, default: t.ink-muted)),
  ..items.map(f => {
    // Narrow cards: no hyphenation.
    set text(hyphenate: false)
    block(below: t.s3, severity-badge(f.severity))
    if f.evidence != none {
      block(below: t.s1, fit-figure(f.evidence.value, size: t.h2 * 1.2, fill: severity-colors.at(f.severity, default: t.ink)))
      if f.evidence.label != none { block(below: t.s3, text(size: t.eyebrow, fill: t.ink-muted, f.evidence.label)) }
    }
    block(below: t.s2, text(font: t.font-heading, size: t.body * 1.05, weight: 600, fill: t.ink, f.title))
    if f.fix != none { block(text(size: t.small, fill: t.ink-muted, f.fix)) }
  }),
)))

/// Impact against effort as a two by two, with numbered dots and a key.
#let priority-matrix(items) = block(above: t.s5, below: t.s5, breakable: false, wide({
  let numbered = items.enumerate().map(((i, item)) => (n: i + 1, ..item))
  let dot(n, c) = box(circle(radius: 7pt, fill: c, align(center + horizon, text(size: 7.5pt, weight: 700, fill: t.paper, number-width: "tabular", str(n)))))
  let quadrant(name, impact, effort, c) = block(width: 100%, height: 58pt, fill: tint(c, amount: 88%), radius: t.r-sm, inset: t.s3, {
    block(below: t.s3, eyebrow-text(name, fill: c.darken(15%)))
    numbered.filter(i => i.impact == impact and i.effort == effort).map(i => dot(i.n, c)).join(h(4pt))
  })
  let axis(body) = text(size: t.eyebrow, fill: t.ink-muted, body)
  grid(
    columns: (1.15fr, 1fr),
    column-gutter: t.s6,
    grid(
      columns: (auto, 1fr, 1fr),
      column-gutter: t.s2,
      row-gutter: t.s2,
      [], align(center, axis[Low effort]), align(center, axis[High effort]),
      align(horizon, rotate(-90deg, reflow: true, axis[High impact])), quadrant("Quick wins", "high", "low", t.good), quadrant("Big bets", "high", "high", t.accent),
      align(horizon, rotate(-90deg, reflow: true, axis[Low impact])), quadrant("Fill-ins", "low", "low", t.ink-muted), quadrant("Maybe later", "low", "high", t.fair),
    ),
    {
      for i in numbered {
        let c = if i.impact == "high" and i.effort == "low" { t.good } else if i.impact == "high" { t.accent } else if i.effort == "low" { t.ink-muted } else { t.fair }
        block(below: t.s2, grid(columns: (auto, 1fr), column-gutter: t.s3, dot(i.n, c), align(horizon, text(size: t.small, fill: t.ink, i.label))))
      }
    },
  )
}))

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

/// A full-page divider on the cover colour that starts a part of a combined
/// document: a large part number, the eyebrow, the title (a real heading, so
/// it's bookmarked and listed in the contents) and an intro.
#let part-divider(number: none, level: 2, title: none, eyebrow: none, intro: none) = page(
  margin: 0mm,
  header: none,
  footer: none,
  fill: t.cover-bg,
)[
  #set text(fill: t.cover-text, hyphenate: false)
  #pad(left: t.margin-left, right: t.margin-right, top: t.margin-top, bottom: t.margin-bottom)[
    #brand-mark(height: 12pt, dark: true, color: t.cover-text)
    #v(1fr)
    #text(font: t.font-heading, size: t.display * 2, weight: 600, fill: blend(t.cover-text, t.cover-bg, 80%), number)
    #v(t.s4)
    #if eyebrow != none { block(below: t.s4, eyebrow-text(eyebrow, fill: blend(t.cover-text, t.cover-bg, 35%))) }
    #{
      show heading: it => text(font: t.font-heading, size: t.display * 0.85, weight: 600, tracking: -0.02em, fill: t.cover-text, it.body)
      heading(level: level, supplement: [#number], title)
    }
    #if intro != none { block(above: t.s5, width: 80%, text(size: t.h3, fill: blend(t.cover-text, t.cover-bg, 20%), intro)) }
    #v(1.3fr)
  ]
]

// ---------------------------------------------------------------------------
// Contents and back page
// ---------------------------------------------------------------------------

/// The sections with their numbers and pages, linked, on a page of its own.
#let contents(title: none) = {
  block(below: t.s7, wide({
    line(length: 100%, stroke: t.heavy + t.ink)
    v(t.s5)
    grid(columns: (t.side, 1fr), [], text(font: t.font-heading, size: t.h1, weight: 600, tracking: -0.015em, fill: t.heading, if title != none { title } else [Contents]))
  }))
  // Entries are links, but read as a list: no underline.
  show underline: it => it.body
  context {
    let offset = if meta.has-cover { 1 } else { 0 }
    let sel = heading.where(level: meta.section-level, outlined: true)
    let sel = if meta.part-level != none { sel.or(heading.where(level: meta.part-level, outlined: true)) } else { sel }
    for entry in query(sel) {
      let page-number = counter(page).at(entry.location()).first() - offset
      // A part: a heading line over its sections, linked like them.
      if entry.level == meta.part-level {
        block(above: t.s6, below: t.s2, wide(link(entry.location(), grid(
          columns: (t.side, 1fr, auto),
          align: bottom,
          eyebrow-text([Part #section-number(entry)], fill: t.accent),
          text(font: t.font-heading, size: t.h2, weight: 600, fill: t.heading, entry.body),
          text(size: t.body, fill: t.ink-muted, number-width: "tabular", str(page-number)),
        ))))
        continue
      }
      block(above: 0pt, below: 0pt, wide(link(entry.location(), block(width: 100%, inset: (y: t.s3), stroke: (bottom: t.hairline + t.rule), grid(
        columns: (t.side, 1fr, auto),
        align: horizon,
        text(font: t.font-heading, size: t.body, weight: 600, fill: t.accent, number-width: "tabular", section-number(entry)),
        [#text(font: t.font-heading, size: t.h3, weight: 600, fill: t.ink, entry.body) #box(width: 1fr, repeat(text(fill: t.rule)[.#h(3pt)]))],
        text(size: t.body, fill: t.ink-muted, number-width: "tabular", str(page-number)),
      )))))
    }
  }
  pagebreak(weak: true)
}

/// A closing page on the cover colour: the brand mark, an about line and
/// contact details.
#let back-page(about: none, contact: ()) = page(margin: 0mm, header: none, footer: none, fill: t.cover-bg)[
  #set text(fill: t.cover-text, hyphenate: false)
  #pad(left: t.margin-left, right: t.margin-right, top: t.margin-top, bottom: t.margin-bottom)[
    #brand-mark(height: 14pt, dark: true, color: t.cover-text)
    #v(1fr)
    #if about != none { block(width: 80%, below: t.s7, text(size: t.h3, fill: blend(t.cover-text, t.cover-bg, 15%), about)) }
    #if contact.len() > 0 {
      block(width: 100%, inset: (top: t.s4), stroke: (top: t.hairline + blend(t.cover-text, t.cover-bg, 75%)), grid(
        columns: (1fr,) * calc.max(contact.len(), 2),
        column-gutter: t.s5,
        ..contact.map(c => {
          block(below: t.s2, eyebrow-text(c.at(0), fill: blend(t.cover-text, t.cover-bg, 45%)))
          text(size: t.body, c.at(1))
        }),
      ))
    }
  ]
]

// ---------------------------------------------------------------------------
// Figures
// ---------------------------------------------------------------------------

/// An image with an optional caption, in the text column or across the full
/// grid. `path` is `none` when the image couldn't be fetched: a framed
/// placeholder shows the alt text instead.
#let figure-image(path, alt: "", caption: none, full: false) = {
  let body = if path != none {
    block(width: 100%, stroke: t.hairline + t.rule, clip: true, radius: t.r-sm, image(path, width: 100%, alt: if alt == "" { none } else { alt }))
  } else {
    block(width: 100%, height: 48pt, fill: t.surface, radius: t.r-sm, align(center + horizon, text(size: t.small, fill: t.ink-muted)[Image unavailable#if alt != "" [: #alt]]))
  }
  let fig = block(above: t.s6, below: t.s6, breakable: false, {
    body
    if caption != none { block(above: t.s3, text(size: t.small, fill: t.ink-muted, caption)) }
  })
  if full { wide(fig) } else { fig }
}
