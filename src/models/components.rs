//! Schema v2 components: blocks that say what something is (a finding, a
//! score, a step) and leave how it looks to the theme.
//!
//! Every struct rejects unknown fields, so a typo fails validation with the
//! field's name instead of silently disappearing from the PDF. Text fields
//! accept inline markdown (bold, italic, links, code).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub enum Component {
    Opener(Opener),
    Callout(Callout),
    Findings(Findings),
    Steps(Steps),
    Scores(Scores),
    Metrics(Metrics),
    Facts(Facts),
}

impl Component {
    /// The JSON `type` names of every component.
    pub const TYPES: [&'static str; 7] =
        ["opener", "callout", "findings", "steps", "scores", "metrics", "facts"];

    /// Parse a component from its JSON block, with `type` already removed.
    pub fn from_json(kind: &str, value: serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(match kind {
            "opener" => Component::Opener(serde_json::from_value(value)?),
            "callout" => Component::Callout(serde_json::from_value(value)?),
            "findings" => Component::Findings(serde_json::from_value(value)?),
            "steps" => Component::Steps(serde_json::from_value(value)?),
            "scores" => Component::Scores(serde_json::from_value(value)?),
            "metrics" => Component::Metrics(serde_json::from_value(value)?),
            "facts" => Component::Facts(serde_json::from_value(value)?),
            other => return Err(serde::de::Error::custom(format!("unknown component `{other}`"))),
        })
    }

    /// The component as plain lines, for renderers without a design for it
    /// (DOCX).
    pub fn plain_lines(&self) -> Vec<String> {
        match self {
            Component::Opener(o) => {
                let mut lines = vec![o.title.clone()];
                lines.extend(o.intro.clone());
                lines
            }
            Component::Callout(c) => c.title.iter().chain(std::iter::once(&c.body)).cloned().collect(),
            Component::Findings(f) => f
                .items
                .iter()
                .map(|i| {
                    let fix = i.fix.as_deref().map(|f| format!(" {f}")).unwrap_or_default();
                    format!("[{}] {}.{fix}", i.severity.label(), i.title)
                })
                .collect(),
            Component::Steps(s) => s
                .groups()
                .flat_map(|g| {
                    let mut lines: Vec<String> = g.title.into_iter().collect();
                    lines.extend(g.items.into_iter().map(|i| i.title));
                    lines
                })
                .collect(),
            Component::Scores(s) => s
                .items
                .iter()
                .map(|i| format!("{}: {} / {}", i.label, i.value, i.max))
                .collect(),
            Component::Metrics(m) => m.items.iter().map(|i| format!("{}: {}", i.label, i.value)).collect(),
            Component::Facts(f) => f.items.iter().map(|i| format!("{}: {}", i.label, i.value)).collect(),
        }
    }
}

/// How something is going: drives colour.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Good,
    Fair,
    Poor,
    Neutral,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Good => "good",
            Status::Fair => "fair",
            Status::Poor => "poor",
            Status::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Serious,
    Moderate,
    Minor,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Serious => "serious",
            Severity::Moderate => "moderate",
            Severity::Minor => "minor",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::Serious => "Serious",
            Severity::Moderate => "Moderate",
            Severity::Minor => "Minor",
        }
    }
}

/// A section's opening: number, title, an eyebrow above, an intro below and
/// an optional headline figure on the right.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opener {
    pub title: String,
    pub eyebrow: Option<String>,
    pub intro: Option<String>,
    pub stat: Option<Stat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stat {
    pub value: String,
    pub label: Option<String>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    #[default]
    Info,
    Good,
    Warn,
    Critical,
    Brand,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Tone::Info => "info",
            Tone::Good => "good",
            Tone::Warn => "warn",
            Tone::Critical => "critical",
            Tone::Brand => "brand",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Callout {
    #[serde(default)]
    pub tone: Tone,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Findings {
    pub items: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub title: String,
    pub severity: Severity,
    /// How many times it was found.
    pub count: Option<u32>,
    /// Where it was found: page names, or a single entry such as "All 5 pages".
    #[serde(default)]
    pub pages: Vec<String>,
    pub fix: Option<String>,
}

/// Numbered steps, optionally in titled groups ("Quick wins", "Bigger jobs").
/// Give `groups`, or `items` for a single untitled group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    #[serde(default)]
    pub groups: Vec<StepGroup>,
    #[serde(default)]
    pub items: Vec<Step>,
}

impl Steps {
    pub fn groups(&self) -> impl Iterator<Item = StepGroup> + '_ {
        let single = (!self.items.is_empty()).then(|| StepGroup { title: None, items: self.items.clone() });
        single.into_iter().chain(self.groups.iter().cloned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepGroup {
    pub title: Option<String>,
    pub items: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub title: String,
    pub detail: Option<String>,
    /// A short effort label, e.g. "1 hour".
    pub effort: Option<String>,
}

/// A row of score gauges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scores {
    pub items: Vec<Score>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    pub label: String,
    pub value: f64,
    #[serde(default = "default_max")]
    pub max: f64,
    /// A target to mark on the gauge.
    pub benchmark: Option<f64>,
}

fn default_max() -> f64 {
    100.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    pub items: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// The figure as it should read, e.g. "7.7 MB".
    pub value: String,
    pub label: String,
    pub target: Option<String>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub items: Vec<Fact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub label: String,
    pub value: String,
}
