use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClientProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String,
    pub name: String,

    // Branding
    pub colour_primary: String,
    pub colour_secondary: String,
    pub colour_accent: String,
    pub colour_text: String,
    pub colour_background: String,
    pub colour_table_header: String,
    pub colour_table_border: String,
    pub colour_callout_bg: String,

    // Typography
    pub font_heading: String,
    pub font_body: String,
    pub font_mono: String,
    pub font_size_base: f32,
    pub font_size_h1: f32,
    pub font_size_h2: f32,
    pub font_size_h3: f32,
    pub line_height: f32,

    // Layout
    pub page_size: String,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub paragraph_spacing: f32,
    pub section_spacing: f32,

    // Logo
    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub logo_width: f32,
    pub logo_position: String,

    // Cover
    pub cover_enabled: bool,
    pub cover_template: String,
    pub cover_bg_colour: String,
    pub cover_text_colour: String,

    // Header / Footer
    pub header_enabled: bool,
    pub header_template: String,
    pub header_border: bool,
    pub footer_enabled: bool,
    pub footer_template: String,
    pub footer_border: bool,

    // Watermark
    pub watermark_text: Option<String>,
    pub watermark_opacity: f32,

    /// Brand name set in the heading font, used where there's no logo image.
    #[serde(default)]
    #[sqlx(default)]
    pub wordmark: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request body for creating/updating a client profile
#[derive(Debug, Deserialize)]
pub struct CreateClientProfile {
    pub slug: String,
    pub name: String,

    #[serde(default = "default_colour_primary")]
    pub colour_primary: String,
    #[serde(default = "default_colour_secondary")]
    pub colour_secondary: String,
    #[serde(default = "default_colour_accent")]
    pub colour_accent: String,
    #[serde(default = "default_colour_text")]
    pub colour_text: String,
    #[serde(default = "default_colour_background")]
    pub colour_background: String,
    #[serde(default = "default_colour_table_header")]
    pub colour_table_header: String,
    #[serde(default = "default_colour_table_border")]
    pub colour_table_border: String,
    #[serde(default = "default_colour_callout_bg")]
    pub colour_callout_bg: String,

    #[serde(default = "default_font_heading")]
    pub font_heading: String,
    #[serde(default = "default_font_body")]
    pub font_body: String,
    #[serde(default = "default_font_mono")]
    pub font_mono: String,
    #[serde(default = "default_font_size_base")]
    pub font_size_base: f32,
    #[serde(default = "default_font_size_h1")]
    pub font_size_h1: f32,
    #[serde(default = "default_font_size_h2")]
    pub font_size_h2: f32,
    #[serde(default = "default_font_size_h3")]
    pub font_size_h3: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,

    #[serde(default = "default_page_size")]
    pub page_size: String,
    #[serde(default = "default_margin")]
    pub margin_top: f32,
    #[serde(default = "default_margin")]
    pub margin_bottom: f32,
    #[serde(default = "default_margin")]
    pub margin_left: f32,
    #[serde(default = "default_margin")]
    pub margin_right: f32,
    #[serde(default = "default_paragraph_spacing")]
    pub paragraph_spacing: f32,
    #[serde(default = "default_section_spacing")]
    pub section_spacing: f32,

    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    #[serde(default = "default_logo_width")]
    pub logo_width: f32,
    #[serde(default = "default_logo_position")]
    pub logo_position: String,

    #[serde(default)]
    pub cover_enabled: bool,
    #[serde(default = "default_cover_template")]
    pub cover_template: String,
    #[serde(default = "default_cover_bg_colour")]
    pub cover_bg_colour: String,
    #[serde(default = "default_cover_text_colour")]
    pub cover_text_colour: String,

    #[serde(default = "default_true")]
    pub header_enabled: bool,
    #[serde(default = "default_header_template")]
    pub header_template: String,
    #[serde(default = "default_true")]
    pub header_border: bool,
    #[serde(default = "default_true")]
    pub footer_enabled: bool,
    #[serde(default = "default_footer_template")]
    pub footer_template: String,
    #[serde(default = "default_true")]
    pub footer_border: bool,

    pub watermark_text: Option<String>,
    #[serde(default = "default_watermark_opacity")]
    pub watermark_opacity: f32,

    #[serde(default)]
    pub wordmark: Option<String>,
}

/// WCAG contrast ratio between two `#RRGGBB` colours.
pub fn contrast_ratio(a: &str, b: &str) -> Option<f64> {
    fn luminance(hex: &str) -> Option<f64> {
        let hex = hex.strip_prefix('#')?;
        if hex.len() != 6 {
            return None;
        }
        let channel = |i: usize| -> Option<f64> {
            let c = u8::from_str_radix(&hex[i..i + 2], 16).ok()? as f64 / 255.0;
            Some(if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) })
        };
        Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
    }
    let (la, lb) = (luminance(a)?, luminance(b)?);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

impl CreateClientProfile {
    /// Check the profile renders as intended: valid colours with readable
    /// contrast, fonts Typeset has, HTTPS logos and a known cover style.
    /// Returns every problem, one per line.
    pub fn validate(&self) -> Result<(), String> {
        let mut problems = Vec::new();
        let colours = [
            ("colour_primary", &self.colour_primary),
            ("colour_secondary", &self.colour_secondary),
            ("colour_accent", &self.colour_accent),
            ("colour_text", &self.colour_text),
            ("colour_background", &self.colour_background),
            ("colour_table_header", &self.colour_table_header),
            ("colour_table_border", &self.colour_table_border),
            ("colour_callout_bg", &self.colour_callout_bg),
            ("cover_bg_colour", &self.cover_bg_colour),
            ("cover_text_colour", &self.cover_text_colour),
        ];
        for (field, value) in colours {
            if contrast_ratio(value, "#000000").is_none() {
                problems.push(format!("{field}: `{value}` isn't a #RRGGBB colour"));
            }
        }
        // Pairs the theme sets text in, with the WCAG AA minimum for each.
        let pairs = [
            ("colour_text", &self.colour_text, "colour_background", &self.colour_background, 4.5),
            ("colour_primary", &self.colour_primary, "colour_background", &self.colour_background, 4.5),
            ("colour_secondary", &self.colour_secondary, "colour_background", &self.colour_background, 4.5),
            ("colour_text", &self.colour_text, "colour_table_header", &self.colour_table_header, 4.5),
            ("colour_accent", &self.colour_accent, "colour_background", &self.colour_background, 3.0),
            ("cover_text_colour", &self.cover_text_colour, "cover_bg_colour", &self.cover_bg_colour, 4.5),
        ];
        for (fa, a, fb, b, min) in pairs {
            if let Some(ratio) = contrast_ratio(a, b) {
                if ratio < min {
                    problems.push(format!("{fa} on {fb}: contrast {ratio:.1}:1 is below {min}:1"));
                }
            }
        }
        for (field, font) in [("font_heading", &self.font_heading), ("font_body", &self.font_body), ("font_mono", &self.font_mono)] {
            if !crate::renderers::typst::is_known_font(font) {
                problems.push(format!(
                    "{field}: `{font}` isn't available; use Inter, Inter Tight or JetBrains Mono"
                ));
            }
        }
        for (field, url) in [("logo_light_url", &self.logo_light_url), ("logo_dark_url", &self.logo_dark_url)] {
            if let Some(url) = url.as_deref().filter(|u| !u.trim().is_empty()) {
                if !url.starts_with("https://") {
                    problems.push(format!("{field}: logos must be https URLs"));
                }
            }
        }
        if !["editorial", "band", "dark", "minimal", "bold", "plain"].contains(&self.cover_template.as_str()) {
            problems.push(format!(
                "cover_template: `{}` isn't one of editorial, band or dark",
                self.cover_template
            ));
        }
        if problems.is_empty() { Ok(()) } else { Err(problems.join("\n")) }
    }
}

impl ClientProfile {
    /// Default profile used when no client slug is provided
    pub fn default_profile() -> Self {
        let now = chrono::Utc::now();
        ClientProfile {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            slug: "default".to_string(),
            name: "Default".to_string(),
            colour_primary: "#000000".to_string(),
            colour_secondary: "#333333".to_string(),
            colour_accent: "#0066CC".to_string(),
            colour_text: "#1A1A1A".to_string(),
            colour_background: "#FFFFFF".to_string(),
            colour_table_header: "#F5F5F5".to_string(),
            colour_table_border: "#E0E0E0".to_string(),
            colour_callout_bg: "#F8F9FA".to_string(),
            font_heading: "Inter Tight".to_string(),
            font_body: "Inter".to_string(),
            font_mono: "JetBrains Mono".to_string(),
            font_size_base: 10.5,
            font_size_h1: 30.0,
            font_size_h2: 20.0,
            font_size_h3: 14.0,
            line_height: 1.55,
            page_size: "A4".to_string(),
            margin_top: 24.0,
            margin_bottom: 22.0,
            margin_left: 22.0,
            margin_right: 22.0,
            paragraph_spacing: 8.0,
            section_spacing: 16.0,
            logo_light_url: None,
            logo_dark_url: None,
            logo_width: 40.0,
            logo_position: "left".to_string(),
            cover_enabled: false,
            cover_template: "minimal".to_string(),
            cover_bg_colour: "#000000".to_string(),
            cover_text_colour: "#FFFFFF".to_string(),
            header_enabled: true,
            header_template: "logo-left".to_string(),
            header_border: true,
            footer_enabled: true,
            footer_template: "page-numbers".to_string(),
            footer_border: true,
            watermark_text: None,
            watermark_opacity: 0.08,
            wordmark: None,
            created_at: now,
            updated_at: now,
        }
    }
}

// Default value functions for serde
fn default_colour_primary() -> String { "#000000".to_string() }
fn default_colour_secondary() -> String { "#333333".to_string() }
fn default_colour_accent() -> String { "#0066CC".to_string() }
fn default_colour_text() -> String { "#1A1A1A".to_string() }
fn default_colour_background() -> String { "#FFFFFF".to_string() }
fn default_colour_table_header() -> String { "#F5F5F5".to_string() }
fn default_colour_table_border() -> String { "#E0E0E0".to_string() }
fn default_colour_callout_bg() -> String { "#F8F9FA".to_string() }
fn default_font_heading() -> String { "Inter Tight".to_string() }
fn default_font_body() -> String { "Inter".to_string() }
fn default_font_mono() -> String { "JetBrains Mono".to_string() }
fn default_font_size_base() -> f32 { 10.5 }
fn default_font_size_h1() -> f32 { 30.0 }
fn default_font_size_h2() -> f32 { 20.0 }
fn default_font_size_h3() -> f32 { 14.0 }
fn default_line_height() -> f32 { 1.55 }
fn default_page_size() -> String { "A4".to_string() }
fn default_margin() -> f32 { 22.0 }
fn default_paragraph_spacing() -> f32 { 8.0 }
fn default_section_spacing() -> f32 { 16.0 }
fn default_logo_width() -> f32 { 40.0 }
fn default_logo_position() -> String { "left".to_string() }
fn default_cover_template() -> String { "minimal".to_string() }
fn default_cover_bg_colour() -> String { "#000000".to_string() }
fn default_cover_text_colour() -> String { "#FFFFFF".to_string() }
fn default_true() -> bool { true }
fn default_header_template() -> String { "logo-left".to_string() }
fn default_footer_template() -> String { "page-numbers".to_string() }
fn default_watermark_opacity() -> f32 { 0.08 }

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(json: serde_json::Value) -> CreateClientProfile {
        let mut base = serde_json::json!({ "slug": "test", "name": "Test" });
        for (k, v) in json.as_object().unwrap() {
            base[k] = v.clone();
        }
        serde_json::from_value(base).unwrap()
    }

    #[test]
    fn the_defaults_are_valid() {
        profile(serde_json::json!({})).validate().unwrap();
    }

    #[test]
    fn contrast_follows_wcag() {
        assert!((contrast_ratio("#000000", "#FFFFFF").unwrap() - 21.0).abs() < 0.01);
        assert!((contrast_ratio("#777777", "#FFFFFF").unwrap() - 4.48).abs() < 0.01);
        assert_eq!(contrast_ratio("blue", "#FFFFFF"), None);
    }

    #[test]
    fn rejects_unreadable_colours_unknown_fonts_and_plain_http_logos() {
        let err = profile(serde_json::json!({
            "colour_text": "#BBBBBB",
            "colour_accent": "nope",
            "font_body": "Comic Sans",
            "logo_light_url": "http://example.com/logo.png",
            "cover_template": "fancy"
        }))
        .validate()
        .unwrap_err();
        assert!(err.contains("colour_text on colour_background"), "{err}");
        assert!(err.contains("colour_accent: `nope`"), "{err}");
        assert!(err.contains("font_body: `Comic Sans`"), "{err}");
        assert!(err.contains("logo_light_url"), "{err}");
        assert!(err.contains("cover_template: `fancy`"), "{err}");
    }
}
