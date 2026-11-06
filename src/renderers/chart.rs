//! Charts drawn by kit's renderer (kit-render: the same code as `<kit-chart>`
//! on the web), embedded in the PDF as SVG in the document's brand.
//!
//! The renderer runs JavaScript in QuickJS, so it's built once per thread
//! and reused. The brand and the column width are set per document by
//! [`set_context`] before the body is written.

use std::cell::RefCell;

use crate::models::components::{Chart, ChartSeries};

/// What a chart needs from the document: kit token values (colours and
/// fonts) and the width it will be shown at.
#[derive(Clone, Default)]
pub struct ChartContext {
    pub theme: serde_json::Map<String, serde_json::Value>,
    /// The text column, in points.
    pub column_pt: f64,
    /// The full content width (text plus margin column), in points.
    pub wide_pt: f64,
}

thread_local! {
    static RENDERER: RefCell<Option<kit_render::Renderer>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<ChartContext> = RefCell::new(ChartContext::default());
}

/// Maps Typeset's profile roles onto kit's token names. Charts sit on the
/// paper; series start from the accent.
#[allow(clippy::too_many_arguments)]
pub fn theme(
    ink: &str,
    muted: &str,
    heading: &str,
    paper: &str,
    surface: &str,
    rule: &str,
    accent: &str,
    accent_soft: &str,
    font_body: &str,
    font_heading: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let pairs = [
        ("bg", paper),
        ("surface", paper),
        ("surface-2", surface),
        ("surface-3", surface),
        ("line", rule),
        ("line-strong", muted),
        ("fg", ink),
        ("fg-2", muted),
        ("fg-3", muted),
        ("accent", accent),
        ("accent-soft", accent_soft),
        ("accent-fg", "#FFFFFF"),
        // The accent, then colours that sit apart from most accents: amber,
        // green, the heading ink, red, the muted ink.
        ("series-1", accent),
        ("series-2", "#D97706"),
        ("series-3", "#1F7A4D"),
        ("series-4", heading),
        ("series-5", "#B42318"),
        ("series-6", muted),
        ("display-weight", "600"),
    ];
    let mut m: serde_json::Map<String, serde_json::Value> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::from(*v)))
        .collect();
    m.insert("font-ui".into(), format!("\"{font_body}\"").into());
    m.insert("font-display".into(), format!("\"{font_heading}\"").into());
    m
}

/// Sets the brand and widths for the document about to be written.
pub fn set_context(ctx: ChartContext) {
    CONTEXT.with(|c| *c.borrow_mut() = ctx);
}

/// The Typst for a chart: the SVG from kit-render, its summary as alt text.
/// If it can't be drawn, a visible note takes its place rather than failing
/// the whole document.
pub fn typst(chart: &Chart) -> String {
    let ctx = CONTEXT.with(|c| c.borrow().clone());
    let width_pt = if chart.wide {
        ctx.wide_pt
    } else {
        ctx.column_pt
    }
    .max(200.0);
    // Draw a little wider than shown, so the chart's 11px labels land at
    // about 8.5pt, a step under the body text.
    let scale = 1.3;
    let mut spec = serde_json::json!({
        "type": chart.chart,
        "width": (width_pt * scale).round(),
        "data": {
            "title": chart.title,
            "labels": chart.labels,
            "series": chart.series.iter().map(|s| serde_json::json!({ "name": s.name, "values": s.values })).collect::<Vec<_>>(),
        },
        "theme": ctx.theme,
        "format": chart.format.clone().unwrap_or_else(|| "number".into()),
        "stacked": chart.stacked,
        "values": chart.values,
    });
    if let Some(c) = &chart.center {
        spec["center"] = c.clone().into();
    }
    if let Some(b) = chart.bins {
        spec["bins"] = b.into();
    }
    if let Some(h) = chart.height {
        spec["height"] = (h * scale).round().into();
    }

    let rendered = RENDERER.with(|r| {
        let mut r = r.borrow_mut();
        if r.is_none() {
            *r = Some(kit_render::Renderer::new()?);
        }
        r.as_ref().expect("just set").render(&spec)
    });
    match rendered {
        Ok(out) => format!(
            "#chart-figure(bytes({}), alt: {}, caption: {}, full: {})\n\n",
            quote(&out.svg),
            quote(&out.summary),
            chart
                .caption
                .as_deref()
                .map(|c| format!("[{}]", super::typst::inline_for_chart(c)))
                .unwrap_or_else(|| "none".into()),
            chart.wide,
        ),
        Err(e) => {
            tracing::warn!(error = %e, kind = %chart.chart, "chart could not be drawn");
            format!(
                "#callout(tone: \"warn\", title: [Chart unavailable])[{}]\n\n",
                quote_content(&e.to_string())
            )
        }
    }
}

/// Reads a ```chart fence: options on the info line (`chart type=line
/// format=compact title="Score" wide`), and the data as a Markdown table,
/// the first column labelling the points and each other column a series. A
/// JSON body (the same shape as the JSON component) works too.
pub fn from_fence(info: &str, body: &str) -> Result<Chart, String> {
    let body = body.trim();
    if body.starts_with('{') {
        let mut v: serde_json::Value =
            serde_json::from_str(body).map_err(|e| format!("chart: {e}"))?;
        if let Some(o) = v.as_object_mut() {
            o.remove("type");
        }
        let c: Chart = serde_json::from_value(v).map_err(|e| format!("chart: {e}"))?;
        c.check()?;
        return Ok(c);
    }
    let opts = options(info);
    let get = |k: &str| opts.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let rows: Vec<Vec<String>> = body
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .filter(|l| !l.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')))
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect()
        })
        .collect();
    let (head, data) = rows
        .split_first()
        .ok_or("chart: the body needs a Markdown table")?;
    if head.len() < 2 {
        return Err("chart: the table needs a label column and at least one value column".into());
    }
    let number = |s: &str| -> Option<f64> {
        let t: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'e' | 'E'))
            .collect();
        if t.is_empty() {
            None
        } else {
            t.parse().ok()
        }
    };
    let c = Chart {
        chart: get("type").unwrap_or_else(|| "bar".into()),
        title: get("title"),
        caption: get("caption"),
        labels: data.iter().map(|r| r[0].clone()).collect(),
        series: (1..head.len())
            .map(|j| ChartSeries {
                name: head[j].clone(),
                values: data
                    .iter()
                    .map(|r| r.get(j).and_then(|c| number(c)))
                    .collect(),
            })
            .collect(),
        format: get("format"),
        stacked: get("stacked").is_some(),
        values: get("values").is_some(),
        center: get("center"),
        bins: get("bins").and_then(|b| b.parse().ok()),
        height: get("height").and_then(|h| h.parse().ok()),
        wide: get("wide").is_some(),
    };
    c.check()?;
    Ok(c)
}

// key=value pairs after "chart", values optionally "quoted"; a bare word is a flag.
fn options(info: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut chars = info.trim().trim_start_matches("chart").chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let key: String =
            std::iter::from_fn(|| chars.next_if(|c| !c.is_whitespace() && *c != '=')).collect();
        if key.is_empty() {
            break;
        }
        let mut value = String::new();
        if chars.next_if_eq(&'=').is_some() {
            if chars.next_if_eq(&'"').is_some() {
                value = std::iter::from_fn(|| chars.next_if(|c| *c != '"')).collect();
                chars.next();
            } else {
                value = std::iter::from_fn(|| chars.next_if(|c| !c.is_whitespace())).collect();
            }
        }
        out.push((key, value));
    }
    out
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn quote_content(s: &str) -> String {
    s.chars()
        .map(|c| {
            if "#*_`$<>@[]\\".contains(c) {
                format!("\\{c}")
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fences_read_options_and_a_table() {
        let c = from_fence(
            "chart type=line format=compact title=\"Score over time\" wide",
            "| Month | Score | Target |\n|---|---|---|\n| Jan | 82 | 90 |\n| Feb | £1,200 | 90 |\n| Mar |  | 90 |",
        )
        .unwrap();
        assert_eq!(c.chart, "line");
        assert_eq!(c.title.as_deref(), Some("Score over time"));
        assert!(c.wide && !c.stacked);
        assert_eq!(c.labels, ["Jan", "Feb", "Mar"]);
        assert_eq!(c.series[0].values, [Some(82.0), Some(1200.0), None]);
        assert_eq!(c.series[1].name, "Target");
    }

    #[test]
    fn fences_take_json_and_report_mistakes() {
        let c = from_fence("chart", r#"{"type":"chart","chart":"donut","labels":["a","b"],"series":[{"name":"n","values":[1,2]}]}"#).unwrap();
        assert_eq!(c.chart, "donut");
        assert!(
            from_fence("chart type=pie", "| a | b |\n|---|---|\n| x | 1 |")
                .unwrap_err()
                .contains("pie")
        );
        assert!(from_fence("chart", "no table").is_err());
    }

    #[test]
    fn charts_become_typst_figures_with_alt_text() {
        set_context(ChartContext {
            theme: theme(
                "#111111",
                "#555555",
                "#000000",
                "#FFFFFF",
                "#F5F5F5",
                "#E0E0E0",
                "#4F46E5",
                "#EEF2FF",
                "Inter",
                "Inter Tight",
            ),
            column_pt: 340.0,
            wide_pt: 450.0,
        });
        let c = from_fence(
            "chart type=bar values caption=\"By category\"",
            "| Category | Score |\n|---|---|\n| SEO | 95 |\n| Content | 64 |",
        )
        .unwrap();
        let out = typst(&c);
        assert!(
            out.starts_with("#chart-figure(bytes(\"<svg xmlns="),
            "{}",
            &out[..80]
        );
        assert!(out.contains("alt: \"Bar chart, 2 points from SEO to Content."));
        assert!(out.contains("#4F46E5"), "the brand's accent");
        assert!(out.contains("&#34;Inter&#34;"), "the brand's body font");
        assert!(out.contains("caption: [By category]"));
    }
}

/// Renders examples/charts.json to PNG pages in KIT_CHART_OUT, in Kritano's
/// colours, for looking at: `KIT_CHART_OUT=/tmp/x cargo test --bin typeset
/// render_chart_example -- --ignored`.
#[cfg(test)]
mod example {
    #[test]
    #[ignore]
    fn render_chart_example() {
        let out = std::env::var("KIT_CHART_OUT").expect("set KIT_CHART_OUT");
        let json =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/charts.json"))
                .unwrap();
        let doc = crate::parser::json::parse(&json).expect("parse");
        let mut profile = crate::models::client_profile::ClientProfile::default_profile();
        profile.colour_accent = "#4F46E5".into();
        profile.colour_primary = "#1E1B4B".into();
        profile.colour_text = "#1B1A33".into();
        profile.colour_secondary = "#4C4B68".into();
        profile.colour_background = "#FFFFFF".into();
        profile.colour_table_header = "#F1F1FA".into();
        profile.colour_table_border = "#E2E1EE".into();
        profile.colour_callout_bg = "#EEF2FF".into();
        profile.cover_bg_colour = "#1E1B4B".into();
        let b = crate::brand::engine::BrandedDocument::prepare(doc, profile).expect("prepare");
        let pages = super::super::typst::render_proof(&b, 110).expect("render");
        std::fs::create_dir_all(&out).unwrap();
        for (i, p) in pages.iter().enumerate() {
            std::fs::write(format!("{out}/page-{i}.png"), p).unwrap();
        }
    }
}
